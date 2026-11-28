//! The tray flyout (Windows): a small window above the notification area with
//! the everyday controls: boost on/off, the gain, and the input and output
//! devices. A setting picks what opens it:
//! - The tray icon, like the system's own flyouts. It closes when it loses
//!   the focus, on Escape, or on another click on the icon.
//! - Windows 11's Quick Settings (the network and volume buttons): it shows
//!   above them without taking the focus, which would close them, and closes
//!   with them.
//!
//! It is a deferred egui viewport that exists from the first frame on and
//! stays hidden until needed. While the main window is hidden, egui still runs
//! frames, but only every 100 ms, so showing, placing and hiding the flyout go
//! straight through Win32 instead of viewport commands.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Once;
use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, Frame, Key, Layout, Margin, Rect, RichText,
    ScrollArea, Sense, Ui, UiBuilder, ViewportBuilder, ViewportClass, ViewportId,
};
use egui_phosphor::regular as icon;

use crate::app::device_labels;
use crate::audio::{DeviceInfo, EngineStatus};
use crate::config::{QuickControls, Settings, DEFAULT_DEVICE_ID};
use crate::controller::{lock, Shared};
use crate::i18n::tr;
use crate::platform::quick_settings::{self as qs, Panels};
use crate::platform::{self, ScreenRect};
use crate::tray::format_db;
use crate::ui::theme::*;
use crate::ui::widgets::*;

/// Lets the window be found among this thread's windows. Never shown.
const TITLE: &str = "Easy MIC Booster quick controls";
/// Logical pixels, like the system flyouts.
const WIDTH: f32 = 360.0;
const PADDING: f32 = 16.0;
/// Gap to the taskbar and the screen edge.
const MARGIN: f32 = 12.0;
/// After opening, a lost focus does not close the flyout: the taskbar can take
/// the focus back right after the click on the icon. Meanwhile the flyout
/// takes it back.
const OPENING: Duration = Duration::from_millis(800);
/// A click on the icon of an open flyout takes the focus first, which closes
/// it; the click itself must not open it again.
const REOPEN_GUARD: Duration = Duration::from_millis(300);
/// How long Quick Settings get to build their panels before they are read.
const SETTLE: Duration = Duration::from_millis(250);
/// How often the panels of Quick Settings are read while they are open.
const FOLLOW_EVERY: Duration = Duration::from_millis(150);
const QUICK_GAINS: [f32; 6] = [0.0, 6.0, 12.0, 18.0, 24.0, 30.0];
const DEVICE_LIST_MAX_H: f32 = 220.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Input,
    Output,
}

#[derive(Clone, Copy)]
struct Anchor {
    area: ScreenRect,
    dpi: u32,
    /// The taskbar is at the top / on the left.
    top: bool,
    left: bool,
}

/// Where Quick Settings are, read by the thread that follows them.
#[derive(Clone, Copy)]
struct Quick {
    panels: Panels,
    dpi: u32,
    /// Top of the work area of their screen.
    screen_top: i32,
}

#[derive(Default)]
struct State {
    hwnd: isize,
    visible: bool,
    /// Asked to open before the window painted its first frame.
    pending_show: bool,
    painted: bool,
    opened_at: Option<Instant>,
    hidden_at: Option<Instant>,
    anchor: Option<Anchor>,
    expanded: Option<Section>,
    /// Height of the content last frame (logical pixels).
    height: f32,
    /// egui's zoom factor last frame.
    zoom: f32,
    /// Quick Settings are open here (set while they are followed).
    quick: Option<Quick>,
    /// Shown above Quick Settings, rather than from the tray icon.
    with_quick: bool,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(Default::default);
static CTX: OnceLock<egui::Context> = OnceLock::new();
/// The setting: what opens the flyout (`QuickControls` as u8). Kept here so
/// the tray and the hook need no lock to know.
static SOURCE: AtomicU8 = AtomicU8::new(0);
/// A thread follows Quick Settings.
static FOLLOWING: AtomicBool = AtomicBool::new(false);

fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

fn viewport_id() -> ViewportId {
    ViewportId::from_hash_of("tray-flyout")
}

fn source() -> QuickControls {
    match SOURCE.load(Ordering::Relaxed) {
        1 => QuickControls::QuickSettings,
        2 => QuickControls::Nowhere,
        _ => QuickControls::TrayIcon,
    }
}

/// Runs in every frame of the main window (the UI thread): keeps the flyout's
/// window in existence, and watches for Quick Settings opening.
pub fn keep(ctx: &egui::Context, ctrl: &Shared, from: QuickControls) {
    let _ = CTX.set(ctx.clone());
    SOURCE.store(
        match from {
            QuickControls::TrayIcon => 0,
            QuickControls::QuickSettings => 1,
            QuickControls::Nowhere => 2,
        },
        Ordering::Relaxed,
    );
    static HOOK: Once = Once::new();
    if qs::supported() {
        HOOK.call_once(|| qs::watch_opened(on_quick_settings_opened));
    }
    let ctrl = ctrl.clone();
    ctx.show_viewport_deferred(
        viewport_id(),
        ViewportBuilder::default()
            .with_title(TITLE)
            .with_decorations(false)
            .with_resizable(false)
            .with_taskbar(false)
            .with_always_on_top()
            .with_visible(false)
            .with_inner_size([WIDTH, 300.0]),
        move |ui, class| {
            if class == ViewportClass::Deferred {
                frame(ui, &ctrl);
            }
        },
    );
}

/// What to do to the window. Win32 calls run only after the state's lock is
/// released: moving, sizing or showing the window makes Windows send it
/// messages right away, eframe then paints it synchronously, and that frame
/// needs the state again (a deadlock on the first click on the tray icon).
/// A click on the icon can also arrive while a Win32 call waits on another
/// process (`SetForegroundWindow`), and needs the state too.
enum Win {
    Place(isize, ScreenRect),
    Show(isize),
    /// Shown without the focus (above Quick Settings).
    ShowInactive(isize),
    Hide(isize),
    NoActivate(isize, bool),
}

fn apply(ops: Vec<Win>) {
    for op in ops {
        match op {
            Win::Place(hwnd, rect) => {
                if platform::window_rect(hwnd) != Some(rect) {
                    platform::place_popup(hwnd, rect);
                }
            }
            Win::Show(hwnd) => platform::show_popup(hwnd),
            Win::ShowInactive(hwnd) => qs::show_inactive(hwnd),
            Win::Hide(hwnd) => platform::hide_popup(hwnd),
            Win::NoActivate(hwnd, on) => qs::set_no_activate(hwnd, on),
        }
    }
}

fn repaint() {
    if let Some(ctx) = CTX.get() {
        ctx.request_repaint_of(viewport_id());
    }
}

/// Whether a click on the tray icon opens the flyout (otherwise it opens the
/// main window).
pub fn opens_from_tray() -> bool {
    source() == QuickControls::TrayIcon
}

// ---- above Quick Settings ------------------------------------------------------

/// Quick Settings just opened (the foreground hook, on the UI thread).
fn on_quick_settings_opened() {
    if source() != QuickControls::QuickSettings || FOLLOWING.swap(true, Ordering::Relaxed) {
        return;
    }
    // Its own thread: COM, and reading the panels takes a while.
    let spawned = std::thread::Builder::new()
        .name("quick-settings".into())
        .spawn(|| {
            if let Err(e) = follow_quick_settings() {
                eprintln!("quick settings: {e}");
            }
            state().quick = None;
            repaint();
            FOLLOWING.store(false, Ordering::Relaxed);
        });
    if spawned.is_err() {
        FOLLOWING.store(false, Ordering::Relaxed);
    }
}

/// Reads where Quick Settings are until they close, for the flyout's frames.
fn follow_quick_settings() -> windows::core::Result<()> {
    qs::init_com()?;
    let reader = qs::Reader::new()?;
    // The top of the panels and the cards on them, kept while the pointer is
    // over the flyout: a card leaving must not move the flyout under it.
    let mut cards_top: Option<i32> = None;
    // They become the foreground window a moment before their window shows,
    // and build their panels after that. Asked meanwhile, UI Automation
    // stalls for seconds; a moment later it answers in milliseconds.
    let started = Instant::now();
    while !qs::is_open() {
        if started.elapsed() > OPENING {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(SETTLE);
    while source() == QuickControls::QuickSettings {
        let Some(panels) = reader.read() else { break };
        let (area, dpi) = platform::work_area_at(panels.left, panels.bottom)
            .unwrap_or((ScreenRect::default(), 96));
        let gap = (2.0 * MARGIN * dpi as f32 / 96.0).round() as i32;
        let hwnd = state().hwnd;
        let held = cards_top
            .filter(|_| hwnd != 0 && qs::pointer_over(hwnd))
            .map(|t| t.min(panels.top));
        let top = held.unwrap_or_else(|| qs::with_other_cards(panels, gap).top);
        cards_top = Some(top);
        state().quick = Some(Quick {
            panels: Panels { top, ..panels },
            dpi,
            screen_top: area.top,
        });
        repaint();
        std::thread::sleep(FOLLOW_EVERY);
    }
    Ok(())
}

/// Above the panels, right edges in line, as far from them as the flyout is
/// from the taskbar; to their left when the screen is too short for that.
fn by_panels(q: &Quick, width: i32, height: i32, margin: i32) -> (i32, i32) {
    let above = q.panels.top - margin - height;
    if above >= q.screen_top + margin {
        (q.panels.right - width, above)
    } else {
        (q.panels.left - margin - width, q.panels.bottom - height)
    }
}

/// A left click on the tray icon at (x, y), in physical pixels.
pub fn toggle(x: i32, y: i32) {
    let ops = {
        let mut st = state();
        toggle_locked(&mut st, x, y)
    };
    apply(ops);
    repaint();
}

fn toggle_locked(st: &mut State, x: i32, y: i32) -> Vec<Win> {
    if st.visible {
        if st.opened_at.is_some_and(|t| t.elapsed() < OPENING) {
            // A double click's second click: stay.
            return (st.hwnd != 0)
                .then_some(Win::Show(st.hwnd))
                .into_iter()
                .collect();
        }
        return close(st);
    }
    if st
        .hidden_at
        .take()
        .is_some_and(|t| t.elapsed() < REOPEN_GUARD)
    {
        return Vec::new();
    }
    let Some((area, dpi)) = platform::work_area_at(x, y) else {
        return Vec::new();
    };
    st.anchor = Some(Anchor {
        area,
        dpi,
        top: y < area.top,
        left: x < area.left,
    });
    st.expanded = None;
    st.opened_at = Some(Instant::now());
    st.visible = true;
    if st.hwnd == 0 {
        if let Some(hwnd) = platform::find_thread_window(TITLE) {
            st.hwnd = hwnd;
            platform::round_corners(hwnd);
        }
    }
    st.with_quick = false;
    if st.hwnd != 0 && st.painted {
        let mut ops = vec![Win::NoActivate(st.hwnd, false)];
        ops.extend(target(st));
        ops.push(Win::Show(st.hwnd));
        ops
    } else {
        // Shown by its next frame.
        st.pending_show = true;
        Vec::new()
    }
}

/// Closes the flyout (the main window opens, the user is done with it).
pub fn hide() {
    let ops = {
        let mut st = state();
        if !st.visible {
            return;
        }
        close(&mut st)
    };
    apply(ops);
    repaint();
}

/// Marks the flyout closed. The caller hides the window and asks for a frame,
/// so the next opening does not show a stale one. Settings are saved by the
/// controller's own saver thread.
fn close(st: &mut State) -> Vec<Win> {
    st.visible = false;
    st.pending_show = false;
    st.hidden_at = Some(Instant::now());
    st.expanded = None;
    st.with_quick = false;
    (st.hwnd != 0)
        .then_some(Win::Hide(st.hwnd))
        .into_iter()
        .collect()
}

/// Where the flyout goes: above Quick Settings (`by_panels`), or from the tray
/// icon at the bottom right of the work area, above the notification area (the
/// corner next to the taskbar when that is at the top or on the left).
fn target(st: &State) -> Option<Win> {
    if st.hwnd == 0 {
        return None;
    }
    let zoom = if st.zoom > 0.0 { st.zoom } else { 1.0 };
    if st.with_quick {
        let q = st.quick?;
        let scale = q.dpi as f32 / 96.0 * zoom;
        let px = |logical: f32| (logical * scale).round() as i32;
        let (margin, width) = (px(MARGIN), px(WIDTH));
        let height = ((st.height * scale).ceil() as i32).max(px(120.0));
        let (left, top) = by_panels(&q, width, height, margin);
        return Some(Win::Place(
            st.hwnd,
            ScreenRect {
                left,
                top,
                right: left + width,
                bottom: top + height,
            },
        ));
    }
    let a = st.anchor?;
    let scale = a.dpi as f32 / 96.0 * zoom;
    let px = |logical: f32| (logical * scale).round() as i32;
    let margin = px(MARGIN);
    let width = px(WIDTH);
    let max_h = (a.area.bottom - a.area.top - 2 * margin).max(px(120.0));
    let height = ((st.height * scale).ceil() as i32).clamp(px(120.0), max_h);
    let left = if a.left {
        a.area.left + margin
    } else {
        a.area.right - width - margin
    };
    let top = if a.top {
        a.area.top + margin
    } else {
        a.area.bottom - height - margin
    };
    Some(Win::Place(
        st.hwnd,
        ScreenRect {
            left,
            top,
            right: left + width,
            bottom: top + height,
        },
    ))
}

// ---- the page ------------------------------------------------------------------

enum Act {
    Boost(bool),
    Gain(f32),
    Input(String),
    Output(String),
    Expand(Option<Section>),
    OpenMain,
}

fn frame(root: &mut Ui, ctrl: &Shared) {
    let (settings, status, devices) = {
        let c = lock(ctrl);
        (c.settings.clone(), c.status(), c.devices())
    };
    let expanded = state().expanded;

    let mut acts = Vec::new();
    let resp = Frame::new()
        .fill(BG)
        .inner_margin(Margin::same(PADDING as i8))
        .show(root, |ui| {
            ui.set_width(WIDTH - 2.0 * PADDING);
            page(ui, &settings, &status, &devices, expanded, &mut acts);
        });
    let height = resp.response.rect.height();
    let escape = root.input(|i| i.key_pressed(Key::Escape));

    let mut open_main = false;
    for a in acts {
        match a {
            Act::Boost(on) => lock(ctrl).set_boost(on),
            Act::Gain(db) => lock(ctrl).edit_sound(|s| s.gain_db = db),
            Act::Input(id) => {
                lock(ctrl).set_input(&id);
                state().expanded = None;
            }
            Act::Output(id) => {
                lock(ctrl).set_output(&id);
                state().expanded = None;
            }
            Act::Expand(s) => state().expanded = s,
            Act::OpenMain => open_main = true,
        }
    }

    let mut ops = Vec::new();
    let mut closed = false;
    {
        let mut st = state();
        st.height = height;
        st.zoom = root.ctx().zoom_factor();
        // Quick Settings opened or closed.
        if st.quick.is_some() && !st.with_quick {
            if st.visible {
                // Open from the tray icon: closed first.
                ops.extend(close(&mut st));
            }
            if st.hwnd == 0 {
                if let Some(hwnd) = platform::find_thread_window(TITLE) {
                    st.hwnd = hwnd;
                    platform::round_corners(hwnd);
                }
            }
            if st.hwnd != 0 {
                st.visible = true;
                st.with_quick = true;
                st.pending_show = false;
                st.opened_at = Some(Instant::now());
                ops.push(Win::NoActivate(st.hwnd, true));
                ops.extend(target(&st));
                ops.push(Win::ShowInactive(st.hwnd));
            }
        } else if st.quick.is_none() && st.with_quick {
            ops.extend(close(&mut st));
            closed = true;
        }
        // From the tray icon, it closes once another window is in front, as
        // the system flyouts do. Asks Windows rather than egui: egui only
        // knows about focus changes it saw a frame for, and the window may
        // never get the focus at all.
        if st.visible && !st.with_quick && st.hwnd != 0 && !st.pending_show {
            let opening = st.opened_at.is_some_and(|t| t.elapsed() < OPENING);
            if platform::is_foreground(st.hwnd) {
                // In front: nothing to do.
            } else if opening {
                // The taskbar can take the focus back right after the click.
                ops.push(Win::Show(st.hwnd));
                root.ctx().request_repaint_after(Duration::from_millis(100));
            } else {
                ops.extend(close(&mut st));
                closed = true;
            }
        }
        if st.visible && (escape || open_main) {
            ops.extend(close(&mut st));
            closed = true;
        }
        if st.visible {
            ops.extend(target(&st));
            if st.pending_show {
                if st.painted && st.hwnd != 0 {
                    st.pending_show = false;
                    ops.push(Win::Show(st.hwnd));
                } else {
                    root.ctx().request_repaint();
                }
            }
            // Status and device changes from elsewhere.
            root.ctx().request_repaint_after(Duration::from_millis(500));
        }
        st.painted = true;
    }
    apply(ops);
    if closed {
        root.ctx().request_repaint();
    }
    if open_main {
        crate::app::show_window();
    }
}

fn page(
    ui: &mut Ui,
    s: &Settings,
    status: &EngineStatus,
    devices: &crate::audio::DeviceList,
    expanded: Option<Section>,
    acts: &mut Vec<Act>,
) {
    // Name, status and the boost switch.
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(
                RichText::new("Easy MIC Booster")
                    .font(bold_font(15.0))
                    .color(TEXT),
            );
            let (color, text) = status_line(status, s.boost_enabled);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(8.0, 12.0), Sense::hover());
                ui.painter().circle_filled(r.center(), 3.5, color);
                ui.label(RichText::new(text).font(font(12.0)).color(TEXT_DIM));
            });
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let mut on = s.boost_enabled;
            if toggle_sized(ui, &mut on, 1.1).changed() {
                acts.push(Act::Boost(on));
            }
            ui.label(
                RichText::new(tr("Tray_Boost"))
                    .font(font(13.0))
                    .color(TEXT_DIM),
            );
        });
    });
    ui.add_space(12.0);

    panel(ui, |ui| gain(ui, s, acts));
    ui.add_space(10.0);

    panel(ui, |ui| {
        let missing = match status {
            EngineStatus::MissingDevice { input, output } => (*input, *output),
            _ => (false, false),
        };
        device_section(
            ui,
            Section::Input,
            icon::MICROPHONE,
            &tr("Tray_Input"),
            &devices.inputs,
            devices.default_input.as_deref(),
            &s.input_device_id,
            &s.input_device_name,
            missing.0,
            expanded,
            acts,
        );
        ui.add_space(2.0);
        device_section(
            ui,
            Section::Output,
            icon::SPEAKER_HIGH,
            &tr("Tray_Output"),
            &devices.outputs,
            devices.default_output.as_deref(),
            &s.output_device_id,
            &s.output_device_name,
            missing.1 || s.output_device_id.is_empty(),
            expanded,
            acts,
        );
    });
    ui.add_space(8.0);

    let r = ui.add(
        egui::Button::new(
            RichText::new(format!("{}  {}", icon::APP_WINDOW, tr("Menu_Open")))
                .font(font(12.5))
                .color(TEXT_DIM),
        )
        .frame(false)
        .min_size(vec2(0.0, 28.0)),
    );
    if r.clicked() {
        acts.push(Act::OpenMain);
    }
}

fn panel(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(SURFACE)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(RADIUS)
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        });
}

fn status_line(status: &EngineStatus, boost: bool) -> (Color32, String) {
    match status {
        EngineStatus::Running { .. } if boost => (GOOD, tr("Status_Live")),
        EngineStatus::Running { .. } => (TEXT_DIM, tr("Status_Bypassed")),
        EngineStatus::Starting => (WARN, tr("Status_Starting")),
        EngineStatus::Idle => (WARN, tr("Status_ChooseOutput")),
        EngineStatus::MissingDevice { input, output } => (
            BAD,
            match (input, output) {
                (true, true) => tr("Status_BothMissing"),
                (true, false) => tr("Status_InputMissing"),
                _ => tr("Status_OutputMissing"),
            },
        ),
        EngineStatus::Error(_) => (BAD, tr("Status_Error")),
    }
}

fn gain(ui: &mut Ui, s: &Settings, acts: &mut Vec<Act>) {
    let g = s.sound.gain_db;
    let max = s.max_gain_db();
    ui.horizontal(|ui| {
        ui.label(dim(tr("Label_Gain")));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let color = if s.boost_enabled { TEXT } else { TEXT_FAINT };
            ui.label(
                RichText::new(format_db(g))
                    .font(bold_font(20.0))
                    .color(color),
            );
        });
    });

    ui.horizontal(|ui| {
        let step = |ui: &mut Ui, label: &str| {
            ui.add(egui::Button::new(label).min_size(vec2(28.0, 26.0)))
                .clicked()
        };
        if step(ui, icon::MINUS) {
            acts.push(Act::Gain(((g - 1.0) * 2.0).round() / 2.0));
        }
        let spacing = ui.spacing().item_spacing.x;
        ui.spacing_mut().slider_width = (ui.available_width() - 28.0 - 2.0 * spacing).max(60.0);
        let mut v = g;
        if ui
            .add(
                egui::Slider::new(&mut v, -30.0..=max)
                    .show_value(false)
                    .step_by(0.5),
            )
            .changed()
        {
            acts.push(Act::Gain(v));
        }
        if step(ui, icon::PLUS) {
            acts.push(Act::Gain(((g + 1.0) * 2.0).round() / 2.0));
        }
    });
    ui.add_space(4.0);

    let values: Vec<f32> = QUICK_GAINS.into_iter().filter(|db| *db <= max).collect();
    ui.horizontal(|ui| {
        let gap = 6.0;
        ui.spacing_mut().item_spacing.x = gap;
        let n = values.len() as f32;
        let w = ((ui.available_width() - gap * (n - 1.0)) / n).floor();
        for db in values {
            let selected = (g - db).abs() < 0.05;
            let (rect, resp) = ui.allocate_exact_size(vec2(w, 28.0), Sense::click());
            let fill = if selected {
                ACCENT
            } else if resp.hovered() {
                SURFACE3
            } else {
                SURFACE2
            };
            ui.painter().rect_filled(rect, 8.0, fill);
            let label = if db == 0.0 {
                "0 dB".to_string()
            } else {
                format!("+{db:.0} dB")
            };
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                font(12.0),
                if selected { Color32::WHITE } else { TEXT_DIM },
            );
            if resp.clicked() && !selected {
                acts.push(Act::Gain(db));
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn device_section(
    ui: &mut Ui,
    section: Section,
    glyph: &str,
    title: &str,
    list: &[DeviceInfo],
    default: Option<&str>,
    current: &str,
    saved_name: &str,
    missing: bool,
    expanded: Option<Section>,
    acts: &mut Vec<Act>,
) {
    let open = expanded == Some(section);
    let (default_label, selected) = device_labels(list, default, current, saved_name);

    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
    if resp.hovered() || open {
        ui.painter().rect_filled(rect, 8.0, SURFACE2);
    }
    ui.painter().text(
        pos2(rect.left() + 16.0, rect.center().y),
        Align2::CENTER_CENTER,
        glyph,
        font(17.0),
        ACCENT_HI,
    );
    ui.painter().text(
        pos2(rect.right() - 14.0, rect.center().y),
        Align2::CENTER_CENTER,
        if open {
            icon::CARET_DOWN
        } else {
            icon::CARET_RIGHT
        },
        font(13.0),
        TEXT_DIM,
    );
    let text_rect = Rect::from_min_max(
        pos2(rect.left() + 36.0, rect.top() + 4.0),
        pos2(rect.right() - 30.0, rect.bottom() - 4.0),
    );
    let mut text = ui.new_child(
        UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    text.spacing_mut().item_spacing.y = 1.0;
    text.add(
        egui::Label::new(RichText::new(title).font(font(11.5)).color(TEXT_DIM)).selectable(false),
    );
    text.add(
        egui::Label::new(RichText::new(&selected).font(font(13.0)).color(if missing {
            BAD
        } else {
            TEXT
        }))
        .truncate()
        .selectable(false),
    );
    if resp.clicked() {
        acts.push(Act::Expand((!open).then_some(section)));
    }

    if !open {
        return;
    }
    let pick = |id: String| match section {
        Section::Input => Act::Input(id),
        Section::Output => Act::Output(id),
    };
    ScrollArea::vertical()
        .id_salt(("flyout-devices", section == Section::Input))
        .max_height(DEVICE_LIST_MAX_H)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            if option_row(ui, &default_label, current == DEFAULT_DEVICE_ID) {
                acts.push(pick(DEFAULT_DEVICE_ID.into()));
            }
            for d in list {
                if option_row(ui, &d.name, current == d.id) {
                    acts.push(pick(d.id.clone()));
                }
            }
        });
}

/// One device in an open list -> clicked (and not already the selected one).
fn option_row(ui: &mut Ui, label: &str, selected: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 6.0, SURFACE3);
    }
    if selected {
        ui.painter().text(
            pos2(rect.left() + 16.0, rect.center().y),
            Align2::CENTER_CENTER,
            icon::CHECK,
            font(14.0),
            ACCENT_HI,
        );
    }
    let text_rect = Rect::from_min_max(
        pos2(rect.left() + 36.0, rect.top()),
        pos2(rect.right() - 8.0, rect.bottom()),
    );
    let mut text = ui.new_child(
        UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    text.add(
        egui::Label::new(RichText::new(label).font(font(12.5)).color(if selected {
            TEXT
        } else {
            TEXT_DIM
        }))
        .truncate()
        .selectable(false),
    );
    resp.clicked() && !selected
}
