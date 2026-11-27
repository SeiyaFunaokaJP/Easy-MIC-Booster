//! The tray flyout (Windows): a small window above the notification area with
//! the everyday controls: boost on/off, the gain, and the input and output
//! devices. A left click on the tray icon opens it, like the system's own
//! flyouts. It closes when it loses the focus, on Escape, or on another click
//! on the icon.
//!
//! It is a deferred egui viewport that exists from the first frame on and
//! stays hidden until needed. While the main window is hidden, egui still runs
//! frames, but only every 100 ms, so showing, placing and hiding the flyout go
//! straight through Win32 instead of viewport commands.

use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, Frame, Key, Layout, Margin, Rect, RichText,
    ScrollArea, Sense, Ui, UiBuilder, ViewportBuilder, ViewportClass, ViewportId,
};
use egui_phosphor::regular as icon;

use crate::app::device_labels;
use crate::audio::{DeviceInfo, EngineStatus};
use crate::config::{Settings, DEFAULT_DEVICE_ID};
use crate::controller::{lock, Shared};
use crate::i18n::tr;
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
/// the focus back right after the click on the icon.
const OPENING: Duration = Duration::from_millis(800);
/// A click on the icon of an open flyout takes the focus first, which closes
/// it; the click itself must not open it again.
const REOPEN_GUARD: Duration = Duration::from_millis(300);
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

#[derive(Default)]
struct State {
    hwnd: isize,
    visible: bool,
    /// Asked to open before the window painted its first frame.
    pending_show: bool,
    painted: bool,
    opened_at: Option<Instant>,
    hidden_at: Option<Instant>,
    had_focus: bool,
    anchor: Option<Anchor>,
    expanded: Option<Section>,
    /// Height of the content last frame (logical pixels).
    height: f32,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(Default::default);
static CTX: OnceLock<(egui::Context, Shared)> = OnceLock::new();

fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

fn viewport_id() -> ViewportId {
    ViewportId::from_hash_of("tray-flyout")
}

/// Runs in every frame of the main window: keeps the flyout's window in existence.
pub fn keep(ctx: &egui::Context, ctrl: &Shared) {
    let _ = CTX.set((ctx.clone(), ctrl.clone()));
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

/// A left click on the tray icon at (x, y), in physical pixels.
pub fn toggle(x: i32, y: i32) {
    let mut st = state();
    if st.visible {
        if st.opened_at.is_some_and(|t| t.elapsed() < OPENING) {
            // A double click's second click: stay.
            if st.hwnd != 0 {
                platform::show_popup(st.hwnd);
            }
            return;
        }
        hide_locked(&mut st);
        drop(st);
        save();
        return;
    }
    if st
        .hidden_at
        .take()
        .is_some_and(|t| t.elapsed() < REOPEN_GUARD)
    {
        return;
    }
    let Some((area, dpi)) = platform::work_area_at(x, y) else {
        return;
    };
    st.anchor = Some(Anchor {
        area,
        dpi,
        top: y < area.top,
        left: x < area.left,
    });
    st.expanded = None;
    st.opened_at = Some(Instant::now());
    st.had_focus = false;
    st.visible = true;
    if st.hwnd == 0 {
        if let Some(hwnd) = platform::find_thread_window(TITLE) {
            st.hwnd = hwnd;
            platform::round_corners(hwnd);
        }
    }
    if st.hwnd != 0 && st.painted {
        place(&st);
        platform::show_popup(st.hwnd);
    } else {
        st.pending_show = true;
    }
    drop(st);
    if let Some((ctx, _)) = CTX.get() {
        ctx.request_repaint_of(viewport_id());
        ctx.request_repaint();
    }
}

/// Closes the flyout (the main window opens, the user is done with it).
pub fn hide() {
    let mut st = state();
    if st.visible {
        hide_locked(&mut st);
        drop(st);
        save();
    }
}

fn hide_locked(st: &mut State) {
    st.visible = false;
    st.pending_show = false;
    st.hidden_at = Some(Instant::now());
    st.expanded = None;
    if st.hwnd != 0 {
        platform::hide_popup(st.hwnd);
    }
    // Repaint while hidden, so the next opening does not show a stale frame.
    if let Some((ctx, _)) = CTX.get() {
        ctx.request_repaint_of(viewport_id());
    }
}

/// Flyout changes are discrete: write them out when it closes.
fn save() {
    if let Some((_, ctrl)) = CTX.get() {
        lock(ctrl).save_now();
    }
}

/// Bottom right of the work area (above the notification area), or the corner
/// next to the taskbar when it is at the top or on the left.
fn place(st: &State) {
    let (Some(a), true) = (st.anchor, st.hwnd != 0) else {
        return;
    };
    let zoom = CTX.get().map_or(1.0, |(ctx, _)| ctx.zoom_factor());
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
    let rect = ScreenRect {
        left,
        top,
        right: left + width,
        bottom: top + height,
    };
    if platform::window_rect(st.hwnd) != Some(rect) {
        platform::place_popup(st.hwnd, rect);
    }
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
    let focused = root.input(|i| i.viewport().focused);

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

    let mut st = state();
    st.height = height;
    let mut closed = false;
    if st.visible {
        match focused {
            Some(true) => st.had_focus = true,
            Some(false) if st.had_focus => {
                if st.opened_at.is_some_and(|t| t.elapsed() < OPENING) {
                    platform::show_popup(st.hwnd);
                } else {
                    hide_locked(&mut st);
                    closed = true;
                }
            }
            _ => {}
        }
    }
    if st.visible && (escape || open_main) {
        hide_locked(&mut st);
        closed = true;
    }
    if st.visible {
        place(&st);
        if st.pending_show {
            if st.painted && st.hwnd != 0 {
                st.pending_show = false;
                platform::show_popup(st.hwnd);
            } else {
                root.ctx().request_repaint();
            }
        }
        // Status and device changes from elsewhere.
        root.ctx().request_repaint_after(Duration::from_millis(500));
    }
    st.painted = true;
    drop(st);
    if closed {
        save();
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
