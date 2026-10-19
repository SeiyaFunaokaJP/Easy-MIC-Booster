//! Main window.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, Frame, Layout, Margin, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui,
};
use egui_phosphor::regular as icon;

use crate::audio::{DeviceInfo, DeviceList, EngineStatus};
use crate::config::{
    BandType, DeviceProfile, EqBand, EqMode, Preset, Settings, Sound, DEFAULT_DEVICE_ID,
    MAX_EQ_POINTS,
};
use crate::controller::{builtin_presets, lock, PresetId, Shared};
use crate::dsp::dynamics::GATE_OFF_DB;
use crate::dsp::{DspShared, FFT_SIZE};
use crate::i18n::{self, tr, trf};
use crate::platform;
use crate::tray::format_db;
use crate::ui::eq_graph::{band_type_label, format_freq, set_band_type, EqGraph};
use crate::ui::theme::*;
use crate::ui::widgets::*;
use crate::update::{self, UpdateState};

struct WindowHooks {
    ctx: egui::Context,
    dsp: Arc<DspShared>,
}

static WINDOW: OnceLock<WindowHooks> = OnceLock::new();

/// Brings the window back (tray click, second instance). Any thread.
pub fn show_window() {
    let Some(w) = WINDOW.get() else { return };
    w.dsp.ui_visible.store(true, Ordering::Relaxed);
    if !platform::show_main_window() {
        w.ctx
            .send_viewport_cmd(egui::ViewportCommand::Visible(true));
        w.ctx
            .send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        w.ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }
    w.ctx.request_repaint();
}

fn hide_window(ctx: &egui::Context, dsp: &DspShared) {
    dsp.ui_visible.store(false, Ordering::Relaxed);
    if platform::SUPPORTS_HIDE {
        platform::hide_main_window();
    } else {
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }
}

pub fn exit_app(ctrl: &Shared) -> ! {
    lock(ctrl).shutdown();
    std::process::exit(0);
}

#[derive(Default)]
struct Meters {
    input: f32,
    input_hold: f32,
    output: f32,
    output_hold: f32,
    hold_time: [f32; 2],
    limiter_db: f32,
    vad: f32,
    gate_db: f32,
    gate_open: bool,
}

impl Meters {
    fn update(&mut self, dsp: &DspShared, dt: f32) {
        let m = &dsp.meters;
        let fall = |v: &mut f32, new: f32, rate: f32| *v = new.max(*v - rate * dt);
        let input = lin_to_meter(m.input_peak.take());
        let output = lin_to_meter(m.output_peak.take());
        fall(&mut self.input, input, 1.6);
        fall(&mut self.output, output, 1.6);
        for (i, (hold, v)) in [
            (&mut self.input_hold, input),
            (&mut self.output_hold, output),
        ]
        .into_iter()
        .enumerate()
        {
            if v >= *hold {
                *hold = v;
                self.hold_time[i] = 1.2;
            } else {
                self.hold_time[i] -= dt;
                if self.hold_time[i] <= 0.0 {
                    *hold = (*hold - 0.6 * dt).max(v);
                }
            }
        }
        let gr = -20.0 * m.limiter_gain.take_min().max(1e-6).log10();
        self.limiter_db = gr.max(self.limiter_db - 12.0 * dt);
        self.vad += (m.vad.load() - self.vad) * (dt * 10.0).min(1.0);
        self.gate_db = m.gate_level_db.load();
        self.gate_open = m.gate_open.load(Ordering::Relaxed);
    }

    fn reset(&mut self) {
        *self = Meters::default();
    }

    /// Whether anything is still moving (signal present or a meter falling).
    fn active(&self) -> bool {
        self.input > 0.01
            || self.output > 0.01
            || self.input_hold > 0.01
            || self.output_hold > 0.01
            || self.vad > 0.02
    }
}

enum Dialog {
    SavePresetAs {
        name: String,
        error: Option<String>,
        focus: bool,
    },
    RenamePreset {
        old: String,
        name: String,
        error: Option<String>,
        focus: bool,
    },
    DeletePreset(String),
    SwitchPreset(PresetId),
    SaveProfile {
        name: String,
        error: Option<String>,
        focus: bool,
    },
    DeleteProfile(String),
    Settings,
}

/// What the frame needs from the controller, copied under one short lock.
struct Snapshot {
    settings: Settings,
    presets: Vec<Preset>,
    profiles: Vec<DeviceProfile>,
    status: EngineStatus,
    devices: DeviceList,
    current: Option<PresetId>,
    current_label: String,
    modified: bool,
    active_profile: Option<String>,
}

pub struct App {
    ctrl: Shared,
    dsp: Arc<DspShared>,
    graph: EqGraph,
    meters: Meters,
    spectrum: Vec<f32>,
    spectrum_version: u64,
    dialog: Option<Dialog>,
    update: Arc<Mutex<UpdateState>>,
    autostart: bool,
    logo: Option<egui::TextureHandle>,
    fonts_lang: String,
    last_frame: Instant,
    error: Option<(String, Instant)>,
    allow_close: bool,
    /// `--minimized`: eframe shows the window after its first frame anyway, so hide it then.
    hide_frames: u8,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, ctrl: Shared, start_hidden: bool) -> Self {
        #[cfg(windows)]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(h) = cc.window_handle() {
                if let RawWindowHandle::Win32(w) = h.as_raw() {
                    platform::set_main_window(w.hwnd.get());
                }
            }
        }
        let ctx = cc.egui_ctx.clone();
        let dsp = lock(&ctrl).dsp().clone();
        dsp.ui_visible.store(!start_hidden, Ordering::Relaxed);
        let _ = WINDOW.set(WindowHooks {
            ctx: ctx.clone(),
            dsp: dsp.clone(),
        });
        if start_hidden && !platform::SUPPORTS_HIDE {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        apply_style(&ctx);
        let lang = i18n::current().to_string();
        install_fonts(&ctx, &lang);

        let update = Arc::new(Mutex::new(UpdateState::Idle));
        if lock(&ctrl).settings.check_updates {
            let c = ctx.clone();
            update::check(update.clone(), Arc::new(move || c.request_repaint()));
        }
        platform::refresh_autostart_path();

        App {
            dsp,
            graph: EqGraph::default(),
            meters: Meters::default(),
            spectrum: vec![-120.0; FFT_SIZE / 2],
            spectrum_version: 0,
            dialog: None,
            update,
            autostart: platform::autostart_enabled(),
            logo: load_logo(&ctx),
            fonts_lang: lang,
            last_frame: Instant::now(),
            error: None,
            allow_close: false,
            hide_frames: if start_hidden { 2 } else { 0 },
            ctrl,
        }
    }

    fn snapshot(&self) -> Snapshot {
        let c = lock(&self.ctrl);
        let current = c.current_preset();
        Snapshot {
            settings: c.settings.clone(),
            presets: c.presets.clone(),
            profiles: c.profiles.clone(),
            status: c.status(),
            devices: c.devices(),
            current_label: current
                .as_ref()
                .map(|id| c.preset_label(id))
                .unwrap_or_default(),
            modified: c.preset_modified(),
            active_profile: c.active_profile().map(|p| p.name.clone()),
            current,
        }
    }

    fn report(&mut self, r: Result<(), String>) -> bool {
        match r {
            Ok(()) => true,
            Err(e) => {
                self.error = Some((e, Instant::now()));
                false
            }
        }
    }
}

fn load_logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let img = image::load_from_memory_with_format(
        include_bytes!("../assets/App.ico"),
        image::ImageFormat::Ico,
    )
    .ok()?;
    let img = img
        .resize(64, 64, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let (w, h) = img.dimensions();
    let color = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &img);
    Some(ctx.load_texture("logo", color, egui::TextureOptions::LINEAR))
}

impl eframe::App for App {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        BG.to_normalized_gamma_f32()
    }

    fn ui(&mut self, root: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = &root.ctx().clone();
        // Close button → tray (or exit).
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close {
            if lock(&self.ctrl).settings.close_to_tray {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                hide_window(ctx, &self.dsp);
            } else {
                exit_app(&self.ctrl);
            }
        }

        if self.hide_frames > 0 {
            self.hide_frames -= 1;
            if self.hide_frames == 0 {
                hide_window(ctx, &self.dsp);
            } else {
                ctx.request_repaint();
            }
        }

        let minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        let visible = !minimized && (!platform::SUPPORTS_HIDE || platform::main_window_visible());
        self.dsp.ui_visible.store(visible, Ordering::Relaxed);

        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.1);
        self.last_frame = now;

        let snap = self.snapshot();
        let running = matches!(snap.status, EngineStatus::Running { .. });
        if running {
            self.meters.update(&self.dsp, dt);
            let v = self.dsp.spectrum_version.load(Ordering::Relaxed);
            if v != self.spectrum_version {
                self.spectrum_version = v;
                if let Ok(s) = self.dsp.spectrum.try_lock() {
                    self.spectrum.copy_from_slice(&s);
                }
            }
        } else {
            self.meters.reset();
            self.spectrum.fill(-120.0);
        }

        if self.fonts_lang != i18n::current() {
            self.fonts_lang = i18n::current().to_string();
            install_fonts(ctx, &self.fonts_lang);
        }

        self.header(root, &snap);
        egui::Panel::left("side")
            .resizable(false)
            .exact_size(330.0)
            .frame(Frame::new().fill(BG).inner_margin(Margin {
                left: 16,
                right: 8,
                top: 4,
                bottom: 16,
            }))
            .show(root, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.routing_card(ui, &snap);
                        ui.add_space(12.0);
                        self.presets_card(ui, &snap);
                    });
            });
        egui::CentralPanel::default()
            .frame(Frame::new().fill(BG).inner_margin(Margin {
                left: 8,
                right: 16,
                top: 4,
                bottom: 16,
            }))
            .show(root, |ui| {
                let avail = ui.available_height();
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.sound_card(ui, &snap, avail);
                        ui.add_space(12.0);
                        ui.columns(2, |cols| {
                            self.suppression_card(&mut cols[0], &snap);
                            self.gate_card(&mut cols[1], &snap);
                        });
                    });
            });

        self.dialogs(ctx, &snap);
        self.error_toast(ctx);

        #[cfg(not(target_os = "linux"))]
        crate::tray::refresh();

        if visible {
            if running && (self.meters.active() || self.graph.spectrum_active()) {
                // Smooth meters only while there is something to show.
                ctx.request_repaint_after(Duration::from_millis(33));
            } else if running {
                // Silence: poll the meters twice a second instead of redrawing at 30 fps.
                ctx.request_repaint_after(Duration::from_millis(500));
            } else if matches!(snap.status, EngineStatus::Starting) {
                ctx.request_repaint_after(Duration::from_millis(250));
            }
        }
    }
}

// ---- header -----------------------------------------------------------------------

impl App {
    fn header(&mut self, root: &mut Ui, snap: &Snapshot) {
        egui::Panel::top("header")
            .exact_size(64.0)
            .frame(Frame::new().fill(BG).inner_margin(Margin {
                left: 20,
                right: 20,
                top: 10,
                bottom: 6,
            }))
            .show_separator_line(false)
            .show(root, |ui| {
                ui.horizontal_centered(|ui| {
                    if let Some(logo) = &self.logo {
                        ui.add(egui::Image::new(logo).fit_to_exact_size(vec2(30.0, 30.0)));
                    }
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("Easy MIC Booster")
                                .font(bold_font(18.0))
                                .color(TEXT),
                        );
                        ui.label(faint(format!("v{}", update::current_version())));
                    });
                    ui.add_space(14.0);
                    status_chip(ui, snap);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if icon_button(ui, icon::GEAR_SIX, &tr("Settings_Title")).clicked() {
                            self.dialog = Some(Dialog::Settings);
                        }
                        if let UpdateState::Available(v) = &*self.update.lock().unwrap() {
                            let r = ui.add(
                                egui::Button::new(
                                    RichText::new(format!(
                                        "{}  {}",
                                        icon::ARROW_CIRCLE_UP,
                                        trf("Msg_UpdateAvailable", &[v])
                                    ))
                                    .color(GOOD)
                                    .font(font(12.0)),
                                )
                                .fill(GOOD.linear_multiply(0.12)),
                            );
                            if r.clicked() {
                                platform::open_url(update::RELEASES_URL);
                            }
                        }
                        ui.add_space(10.0);
                        let mut on = snap.settings.boost_enabled;
                        let r = toggle_sized(ui, &mut on, 1.25).on_hover_text(tr("Tip_Boost"));
                        if r.changed() {
                            lock(&self.ctrl).set_boost(on);
                        }
                        let (text, color) = if on {
                            ("BOOST ON", ACCENT_HI)
                        } else {
                            ("BOOST OFF", TEXT_FAINT)
                        };
                        ui.label(RichText::new(text).font(bold_font(14.0)).color(color));
                    });
                });
            });
    }
}

fn status_chip(ui: &mut Ui, snap: &Snapshot) {
    let (color, text, tip) = match &snap.status {
        EngineStatus::Running {
            input_rate,
            latency_ms,
            ..
        } => {
            let mut t = if snap.settings.boost_enabled {
                tr("Status_Live")
            } else {
                tr("Status_Bypassed")
            };
            t += &format!(
                "  ·  {} kHz  ·  ~{:.0} ms",
                *input_rate as f32 / 1000.0,
                latency_ms
            );
            (
                if snap.settings.boost_enabled {
                    GOOD
                } else {
                    TEXT_DIM
                },
                t,
                None,
            )
        }
        EngineStatus::Starting => (WARN, tr("Status_Starting"), None),
        EngineStatus::Idle => (WARN, tr("Status_ChooseOutput"), None),
        EngineStatus::MissingDevice { input, output } => {
            let t = match (input, output) {
                (true, true) => tr("Status_BothMissing"),
                (true, false) => tr("Status_InputMissing"),
                _ => tr("Status_OutputMissing"),
            };
            (BAD, t, Some(tr("Status_MissingHint")))
        }
        EngineStatus::Error(e) => (BAD, tr("Status_Error"), Some(e.clone())),
    };
    let galley = ui.painter().layout_no_wrap(text, font(12.5), TEXT);
    let size = vec2(galley.size().x + 34.0, 28.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, 14.0, color.linear_multiply(0.12));
    ui.painter().rect_stroke(
        rect,
        14.0,
        Stroke::new(1.0, color.linear_multiply(0.35)),
        StrokeKind::Inside,
    );
    let dot = pos2(rect.left() + 14.0, rect.center().y);
    let pulse = if color == GOOD {
        (ui.input(|i| i.time) * 2.0).sin() as f32 * 0.5 + 0.5
    } else {
        0.0
    };
    ui.painter().circle_filled(
        dot,
        4.0 + pulse * 3.0,
        color.linear_multiply(0.25 * (1.0 - pulse)),
    );
    ui.painter().circle_filled(dot, 4.0, color);
    ui.painter().galley(
        pos2(rect.left() + 24.0, rect.center().y - galley.size().y / 2.0),
        galley,
        TEXT,
    );
    if let Some(t) = tip {
        resp.on_hover_text(t);
    }
}

// ---- routing ----------------------------------------------------------------------

impl App {
    fn routing_card(&mut self, ui: &mut Ui, snap: &Snapshot) {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                section_title(ui, icon::PLUGS_CONNECTED, &tr("Group_Device"));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_button(ui, icon::ARROWS_CLOCKWISE, &tr("Btn_Refresh")).clicked() {
                        lock(&self.ctrl).engine.refresh_devices();
                    }
                });
            });
            ui.add_space(4.0);
            let s = &snap.settings;
            let d = &snap.devices;
            let missing = match snap.status {
                EngineStatus::MissingDevice { input, output } => (input, output),
                _ => (false, false),
            };

            ui.label(dim(format!("{}  {}", icon::MICROPHONE, tr("Label_Input"))));
            if let Some(id) = device_combo(
                ui,
                "in",
                &d.inputs,
                d.default_input.as_deref(),
                &s.input_device_id,
                &s.input_device_name,
                missing.0,
            ) {
                lock(&self.ctrl).set_input(&id);
            }
            meter(ui, self.meters.input, self.meters.input_hold, 6.0, false);

            ui.add_space(2.0);
            ui.vertical_centered(|ui| {
                let mut chain = Vec::new();
                if s.boost_enabled {
                    chain.push(format_db(s.sound.gain_db));
                    if s.sound.mode == EqMode::Equalizer && !s.sound.bands.is_empty() {
                        chain.push("EQ".into());
                    }
                }
                if s.sound.noise_suppression {
                    chain.push("AI NS".into());
                }
                if s.sound.noise_gate_db > GATE_OFF_DB {
                    chain.push("Gate".into());
                }
                let text = if chain.is_empty() {
                    tr("Chain_Passthrough")
                } else {
                    chain.join("  ·  ")
                };
                ui.label(faint(format!(
                    "{}  {}  {}",
                    icon::ARROW_DOWN,
                    text,
                    icon::ARROW_DOWN
                )));
            });

            ui.label(dim(format!(
                "{}  {}",
                icon::SPEAKER_HIGH,
                tr("Label_Output")
            )));
            if let Some(id) = device_combo(
                ui,
                "out",
                &d.outputs,
                d.default_output.as_deref(),
                &s.output_device_id,
                &s.output_device_name,
                missing.1 || s.output_device_id.is_empty(),
            ) {
                lock(&self.ctrl).set_output(&id);
            }
            let limiting = self.meters.limiter_db > 0.5;
            meter(
                ui,
                self.meters.output,
                self.meters.output_hold,
                6.0,
                limiting,
            );
            if limiting {
                ui.label(
                    RichText::new(trf(
                        "Label_Limiting",
                        &[&format!("{:.1}", self.meters.limiter_db)],
                    ))
                    .color(WARN)
                    .font(font(11.0)),
                );
            }
            if s.output_device_id.is_empty() {
                ui.label(
                    RichText::new(tr("Hint_ChooseOutput"))
                        .color(WARN)
                        .font(font(11.5)),
                );
            } else if s.output_device_id == DEFAULT_DEVICE_ID
                || s.output_device_id == d.default_output.clone().unwrap_or_default()
            {
                ui.label(
                    RichText::new(format!(
                        "{}  {}",
                        icon::WARNING,
                        tr("Hint_OutputIsSpeakers")
                    ))
                    .color(WARN)
                    .font(font(11.0)),
                );
            }

            ui.add_space(6.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(dim(tr("Label_DeviceProfile")));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let text = snap
                        .active_profile
                        .clone()
                        .unwrap_or_else(|| tr("Profile_None"));
                    egui::ComboBox::from_id_salt("profile")
                        .width(ui.available_width())
                        .selected_text(text)
                        .show_ui(ui, |ui| {
                            for p in &snap.profiles {
                                let sel = snap.active_profile.as_deref() == Some(&p.name);
                                let r = ui.selectable_label(sel, &p.name).on_hover_text(format!(
                                    "{} {}\n{} {}",
                                    icon::MICROPHONE,
                                    p.input_device_name,
                                    icon::SPEAKER_HIGH,
                                    p.output_device_name
                                ));
                                if r.clicked() {
                                    lock(&self.ctrl).apply_profile(&p.name);
                                }
                            }
                            if !snap.profiles.is_empty() {
                                ui.separator();
                            }
                            if ui
                                .button(format!(
                                    "{}  {}",
                                    icon::FLOPPY_DISK,
                                    tr("Profile_SaveCurrent")
                                ))
                                .clicked()
                            {
                                self.dialog = Some(Dialog::SaveProfile {
                                    name: snap.active_profile.clone().unwrap_or_default(),
                                    error: None,
                                    focus: true,
                                });
                            }
                            if let Some(name) = &snap.active_profile {
                                if ui
                                    .button(
                                        RichText::new(format!(
                                            "{}  {}",
                                            icon::TRASH,
                                            trf("Profile_Delete", &[name])
                                        ))
                                        .color(BAD),
                                    )
                                    .clicked()
                                {
                                    self.dialog = Some(Dialog::DeleteProfile(name.clone()));
                                }
                            }
                        });
                });
            });
        });
    }
}

fn device_combo(
    ui: &mut Ui,
    salt: &str,
    list: &[DeviceInfo],
    default: Option<&str>,
    current: &str,
    saved_name: &str,
    missing: bool,
) -> Option<String> {
    let default_label = match default.and_then(|id| list.iter().find(|d| d.id == id)) {
        Some(d) => trf("Device_Default", &[&d.name]),
        None => tr("Device_Default_None"),
    };
    let selected = if current == DEFAULT_DEVICE_ID {
        default_label.clone()
    } else if let Some(d) = list.iter().find(|d| d.id == current) {
        d.name.clone()
    } else if current.is_empty() {
        tr("Msg_DeviceUnselected")
    } else {
        trf(
            "Device_Disconnected",
            &[&if saved_name.is_empty() {
                current
            } else {
                saved_name
            }],
        )
    };
    let mut picked = None;
    let r = egui::ComboBox::from_id_salt(salt)
        .width(ui.available_width())
        .height(360.0)
        .selected_text(RichText::new(selected).color(if missing { BAD } else { TEXT }))
        .truncate()
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(current == DEFAULT_DEVICE_ID, default_label)
                .clicked()
            {
                picked = Some(DEFAULT_DEVICE_ID.to_string());
            }
            ui.separator();
            for d in list {
                if ui.selectable_label(current == d.id, &d.name).clicked() {
                    picked = Some(d.id.clone());
                }
            }
        });
    if missing {
        ui.painter().rect_stroke(
            r.response.rect.expand(1.0),
            8.0,
            Stroke::new(1.5, BAD),
            StrokeKind::Outside,
        );
    }
    picked.filter(|p| p != current)
}

// ---- presets ----------------------------------------------------------------------

impl App {
    fn presets_card(&mut self, ui: &mut Ui, snap: &Snapshot) {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                section_title(ui, icon::BOOKMARKS_SIMPLE, &tr("Group_Presets"));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_button(ui, icon::FOLDER_OPEN, &tr("Tip_OpenPresetFolder")).clicked() {
                        platform::open_path(&lock(&self.ctrl).store.presets_dir());
                    }
                });
            });

            // Current state banner.
            Frame::new()
                .fill(SURFACE2)
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let name = if snap.current.is_some() {
                            snap.current_label.clone()
                        } else {
                            tr("Preset_Unsaved")
                        };
                        ui.label(RichText::new(name).font(bold_font(14.0)).color(TEXT));
                        if snap.modified {
                            badge(ui, &tr("Preset_Modified"), WARN);
                        }
                    });
                    ui.horizontal(|ui| {
                        let is_user = matches!(snap.current, Some(PresetId::User(_)));
                        if is_user
                            && snap.modified
                            && primary_button(
                                ui,
                                format!("{}  {}", icon::FLOPPY_DISK, tr("Btn_Save")),
                            )
                            .clicked()
                        {
                            if let Some(PresetId::User(n)) = &snap.current {
                                let r = lock(&self.ctrl).save_preset_as(n);
                                self.report(r);
                            }
                        }
                        let save_as = ui.button(format!("{}  {}", icon::PLUS, tr("Btn_SaveAsNew")));
                        if save_as.clicked() {
                            self.dialog = Some(Dialog::SavePresetAs {
                                name: String::new(),
                                error: None,
                                focus: true,
                            });
                        }
                        if snap.modified
                            && ui
                                .button(format!(
                                    "{}  {}",
                                    icon::ARROW_COUNTER_CLOCKWISE,
                                    tr("Btn_Revert")
                                ))
                                .on_hover_text(tr("Tip_Revert"))
                                .clicked()
                        {
                            lock(&self.ctrl).revert_preset();
                        }
                    });
                });
            ui.add_space(6.0);

            ui.label(faint(tr("Presets_Mine")));
            if snap.presets.is_empty() {
                ui.label(faint(tr("Presets_Empty")));
            }
            for p in &snap.presets {
                let id = PresetId::User(p.name.clone());
                let summary = preset_summary(&p.to_sound(&snap.settings.sound));
                if let Some(action) = preset_row(
                    ui,
                    &p.name,
                    &summary,
                    snap.current.as_ref() == Some(&id),
                    snap.modified,
                    true,
                ) {
                    match action {
                        RowAction::Apply => self.request_preset(snap, id),
                        RowAction::Rename => {
                            self.dialog = Some(Dialog::RenamePreset {
                                old: p.name.clone(),
                                name: p.name.clone(),
                                error: None,
                                focus: true,
                            })
                        }
                        RowAction::Overwrite => {
                            let r = lock(&self.ctrl).save_preset_as(&p.name);
                            self.report(r);
                        }
                        RowAction::Delete => {
                            self.dialog = Some(Dialog::DeletePreset(p.name.clone()))
                        }
                    }
                }
            }
            ui.add_space(6.0);
            ui.label(faint(tr("Presets_Builtin")));
            for (bid, key, p) in builtin_presets() {
                let id = PresetId::Builtin(bid);
                let summary = preset_summary(&p.to_sound(&snap.settings.sound));
                if let Some(RowAction::Apply) = preset_row(
                    ui,
                    &tr(key),
                    &summary,
                    snap.current.as_ref() == Some(&id),
                    snap.modified,
                    false,
                ) {
                    self.request_preset(snap, id);
                }
            }
        });
    }

    /// Switching away from a modified user preset asks first instead of
    /// silently dropping the edits (the v1.x combo box did).
    fn request_preset(&mut self, snap: &Snapshot, id: PresetId) {
        if snap.current.as_ref() == Some(&id) && !snap.modified {
            return;
        }
        if snap.modified && matches!(snap.current, Some(PresetId::User(_))) {
            self.dialog = Some(Dialog::SwitchPreset(id));
        } else {
            lock(&self.ctrl).apply_preset(&id);
        }
    }
}

fn preset_summary(s: &Sound) -> String {
    let mut parts = vec![format_db(s.gain_db)];
    if s.mode == EqMode::Equalizer && !s.bands.is_empty() {
        parts.push(format!("EQ ×{}", s.bands.len()));
    }
    if s.noise_gate_db > GATE_OFF_DB {
        parts.push(format!("Gate {:.0} dB", s.noise_gate_db));
    }
    parts.join(" · ")
}

enum RowAction {
    Apply,
    Rename,
    Overwrite,
    Delete,
}

fn preset_row(
    ui: &mut Ui,
    name: &str,
    summary: &str,
    selected: bool,
    modified: bool,
    user: bool,
) -> Option<RowAction> {
    let mut action = None;
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
    let hovered = resp.hovered();
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, 9.0, ACCENT.linear_multiply(0.18));
        p.rect_stroke(
            rect,
            9.0,
            Stroke::new(1.0, ACCENT.linear_multiply(0.6)),
            StrokeKind::Inside,
        );
    } else if hovered {
        p.rect_filled(rect, 9.0, SURFACE2);
    }
    let ic = if user { icon::WAVEFORM } else { icon::SPARKLE };
    p.text(
        pos2(rect.left() + 14.0, rect.center().y),
        Align2::CENTER_CENTER,
        ic,
        font(15.0),
        if selected { ACCENT_HI } else { TEXT_DIM },
    );
    let mut title = name.to_string();
    if selected && modified {
        title += " •";
    }
    let text_w = rect.width() - 70.0;
    let galley = p.layout(title, bold_font(13.0), TEXT, text_w);
    p.galley(pos2(rect.left() + 30.0, rect.top() + 4.0), galley, TEXT);
    let galley = p.layout(summary.to_string(), font(11.0), TEXT_FAINT, text_w);
    p.galley(
        pos2(rect.left() + 30.0, rect.top() + 21.0),
        galley,
        TEXT_FAINT,
    );
    if resp.clicked() {
        action = Some(RowAction::Apply);
    }
    if user {
        let menu_rect =
            Rect::from_center_size(pos2(rect.right() - 18.0, rect.center().y), vec2(28.0, 28.0));
        if hovered || selected {
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(menu_rect));
            child.menu_button(RichText::new(icon::DOTS_THREE).font(font(16.0)), |ui| {
                ui.set_min_width(190.0);
                if ui
                    .button(format!("{}  {}", icon::PENCIL_SIMPLE, tr("Preset_Rename")))
                    .clicked()
                {
                    action = Some(RowAction::Rename);
                    ui.close();
                }
                if ui
                    .button(format!("{}  {}", icon::FLOPPY_DISK, tr("Preset_Overwrite")))
                    .clicked()
                {
                    action = Some(RowAction::Overwrite);
                    ui.close();
                }
                ui.separator();
                if ui
                    .button(
                        RichText::new(format!("{}  {}", icon::TRASH, tr("Btn_Delete"))).color(BAD),
                    )
                    .clicked()
                {
                    action = Some(RowAction::Delete);
                    ui.close();
                }
            });
        }
        resp.context_menu(|ui| {
            if ui
                .button(format!("{}  {}", icon::PENCIL_SIMPLE, tr("Preset_Rename")))
                .clicked()
            {
                action = Some(RowAction::Rename);
                ui.close();
            }
            if ui
                .button(format!("{}  {}", icon::FLOPPY_DISK, tr("Preset_Overwrite")))
                .clicked()
            {
                action = Some(RowAction::Overwrite);
                ui.close();
            }
            if ui
                .button(RichText::new(format!("{}  {}", icon::TRASH, tr("Btn_Delete"))).color(BAD))
                .clicked()
            {
                action = Some(RowAction::Delete);
                ui.close();
            }
        });
    }
    action
}

// ---- sound --------------------------------------------------------------------------

impl App {
    fn sound_card(&mut self, ui: &mut Ui, snap: &Snapshot, avail: f32) {
        let s = &snap.settings;
        card(ui, |ui| {
            ui.horizontal(|ui| {
                section_title(ui, icon::SLIDERS_HORIZONTAL, &tr("Group_Sound"));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let labels = [tr("Mode_Simple"), tr("Mode_Equalizer")];
                    let sel = if s.sound.mode == EqMode::Simple { 0 } else { 1 };
                    if let Some(i) = segmented(ui, &labels, sel) {
                        lock(&self.ctrl).set_mode(if i == 0 {
                            EqMode::Simple
                        } else {
                            EqMode::Equalizer
                        });
                    }
                });
            });

            // Gain.
            ui.horizontal(|ui| {
                ui.label(dim(tr("Label_Gain")));
                ui.label(
                    RichText::new(format_db(s.sound.gain_db))
                        .font(bold_font(24.0))
                        .color(if s.boost_enabled { TEXT } else { TEXT_FAINT }),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut g = s.sound.gain_db;
                    let step = |ui: &mut Ui, label: &str, d: f32, g: &mut f32| {
                        if ui
                            .add(egui::Button::new(label).min_size(vec2(34.0, 28.0)))
                            .clicked()
                        {
                            *g = ((*g + d) * 2.0).round() / 2.0;
                            true
                        } else {
                            false
                        }
                    };
                    let mut changed = step(ui, "+1", 1.0, &mut g);
                    changed |= step(ui, "−1", -1.0, &mut g);
                    ui.add_space(6.0);
                    ui.spacing_mut().slider_width = (ui.available_width() - 8.0).max(80.0);
                    let r = ui.add(
                        egui::Slider::new(&mut g, -30.0..=s.max_gain_db())
                            .show_value(false)
                            .step_by(0.1),
                    );
                    changed |= r.changed();
                    if changed {
                        lock(&self.ctrl).edit_sound(|x| x.gain_db = g);
                    }
                });
            });

            let graph_h = (avail - 360.0).clamp(220.0, 460.0);
            let spectrum = matches!(snap.status, EngineStatus::Running { .. })
                .then_some(self.spectrum.as_slice());
            if let Some(new) = self.graph.show(
                ui,
                &s.sound,
                s.boost_enabled,
                s.max_gain_db(),
                spectrum,
                graph_h,
            ) {
                lock(&self.ctrl).edit_sound(|x| *x = new);
            }

            if s.sound.mode == EqMode::Equalizer {
                self.eq_toolbar(ui, &s.sound);
            } else {
                ui.label(faint(tr("Hint_SimpleMode")));
            }
        });
    }

    fn eq_toolbar(&mut self, ui: &mut Ui, sound: &Sound) {
        let mut bands = sound.bands.clone();
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            if let Some(i) = self.graph.selected.filter(|&i| i < bands.len()) {
                let color = POINT_COLORS[i % POINT_COLORS.len()];
                ui.label(
                    RichText::new(format!("#{}", i + 1))
                        .font(bold_font(13.0))
                        .color(color),
                );
                let b = &mut bands[i];
                egui::ComboBox::from_id_salt("bandtype")
                    .width(120.0)
                    .selected_text(band_type_label(b.band_type))
                    .show_ui(ui, |ui| {
                        for t in BandType::ALL {
                            if ui
                                .selectable_label(b.band_type == t, band_type_label(t))
                                .clicked()
                            {
                                set_band_type(b, t);
                                changed = true;
                            }
                        }
                    });
                let mut f = b.frequency;
                let speed = (f * 0.01).max(0.5);
                let r = ui.add(
                    egui::DragValue::new(&mut f)
                        .range(20.0..=20_000.0)
                        .speed(speed)
                        .custom_formatter(|v, _| format_freq(v as f32))
                        .custom_parser(parse_freq),
                );
                if r.changed() {
                    b.frequency = f;
                    changed = true;
                }
                if b.band_type.has_gain() {
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut b.gain)
                                .range(-24.0..=24.0)
                                .speed(0.1)
                                .suffix(" dB")
                                .fixed_decimals(1),
                        )
                        .changed();
                }
                if b.band_type.has_q() {
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut b.q)
                                .range(0.1..=12.0)
                                .speed(0.01)
                                .prefix("Q ")
                                .fixed_decimals(2),
                        )
                        .changed();
                }
                if icon_button(ui, icon::TRASH, &tr("Eq_DeletePoint")).clicked() {
                    bands.remove(i);
                    self.graph.selected = None;
                    changed = true;
                }
            } else {
                ui.label(faint(tr("Help_Text")));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_enabled(
                        !bands.is_empty(),
                        egui::Button::new(format!(
                            "{}  {}",
                            icon::ARROW_COUNTER_CLOCKWISE,
                            tr("Eq_Reset")
                        )),
                    )
                    .clicked()
                {
                    bands.clear();
                    self.graph.selected = None;
                    changed = true;
                }
                if ui
                    .add_enabled(
                        bands.len() < MAX_EQ_POINTS,
                        egui::Button::new(format!("{}  {}", icon::PLUS, tr("Btn_AddPoint"))),
                    )
                    .clicked()
                {
                    bands.push(EqBand::peaking(1000.0, 0.0, 1.0));
                    self.graph.selected = Some(bands.len() - 1);
                    changed = true;
                }
            });
        });
        if changed {
            lock(&self.ctrl).edit_sound(|s| s.bands = bands);
        }
    }

    fn suppression_card(&mut self, ui: &mut Ui, snap: &Snapshot) {
        let s = &snap.settings.sound;
        card(ui, |ui| {
            ui.horizontal(|ui| {
                section_title(ui, icon::SPARKLE, &tr("Group_NoiseSuppression"));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut on = s.noise_suppression;
                    if toggle(ui, &mut on).changed() {
                        lock(&self.ctrl).edit_sound(|x| x.noise_suppression = on);
                    }
                });
            });
            ui.label(faint(tr("Help_NoiseSuppression")));
            ui.add_enabled_ui(s.noise_suppression, |ui| {
                ui.horizontal(|ui| {
                    ui.label(dim(tr("Label_Strength")));
                    let mut v = s.suppression_strength * 100.0;
                    ui.spacing_mut().slider_width = (ui.available_width() - 60.0).max(60.0);
                    if ui
                        .add(
                            egui::Slider::new(&mut v, 0.0..=100.0)
                                .suffix(" %")
                                .fixed_decimals(0),
                        )
                        .changed()
                    {
                        lock(&self.ctrl).edit_sound(|x| x.suppression_strength = v / 100.0);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(dim(tr("Label_Voice")));
                    let vad = if s.noise_suppression
                        && matches!(snap.status, EngineStatus::Running { .. })
                    {
                        self.meters.vad
                    } else {
                        0.0
                    };
                    meter(ui, vad, 0.0, 6.0, false);
                });
            });
        });
    }

    fn gate_card(&mut self, ui: &mut Ui, snap: &Snapshot) {
        let s = &snap.settings.sound;
        let on = s.noise_gate_db > GATE_OFF_DB;
        card(ui, |ui| {
            ui.horizontal(|ui| {
                section_title(ui, icon::DOOR, &tr("Group_NoiseGate"));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut v = on;
                    if toggle(ui, &mut v).changed() {
                        lock(&self.ctrl)
                            .edit_sound(|x| x.noise_gate_db = if v { -50.0 } else { GATE_OFF_DB });
                    }
                });
            });
            ui.label(faint(tr("Help_NoiseGate")));

            // Level meter with a draggable threshold marker.
            let (rect, resp) =
                ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::click_and_drag());
            let to_x = |db: f32| {
                rect.left() + ((db - GATE_OFF_DB) / -GATE_OFF_DB).clamp(0.0, 1.0) * rect.width()
            };
            let running = matches!(snap.status, EngineStatus::Running { .. });
            let p = ui.painter();
            p.rect_filled(rect, 6.0, Color32::from_rgb(0x0b, 0x0d, 0x12));
            if running && on {
                let fill =
                    Rect::from_min_max(rect.min, pos2(to_x(self.meters.gate_db), rect.max.y));
                p.rect_filled(
                    fill,
                    6.0,
                    if self.meters.gate_open {
                        GOOD.linear_multiply(0.75)
                    } else {
                        TEXT_FAINT
                    },
                );
            }
            if on {
                let x = to_x(s.noise_gate_db);
                p.rect_filled(
                    Rect::from_min_max(pos2(x, rect.top()), rect.max),
                    6.0,
                    BAD.linear_multiply(0.08),
                );
                p.line_segment(
                    [pos2(x, rect.top() - 3.0), pos2(x, rect.bottom() + 3.0)],
                    Stroke::new(2.5, ACCENT_HI),
                );
            }
            if on && (resp.dragged() || resp.clicked()) {
                if let Some(pos) = resp.interact_pointer_pos() {
                    let db = GATE_OFF_DB
                        + ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0) * -GATE_OFF_DB;
                    let db = db.round().clamp(GATE_OFF_DB + 1.0, 0.0);
                    lock(&self.ctrl).edit_sound(|x| x.noise_gate_db = db);
                }
            }
            if on {
                ui.ctx().set_cursor_icon(if resp.hovered() {
                    egui::CursorIcon::ResizeHorizontal
                } else {
                    egui::CursorIcon::Default
                });
            }
            ui.horizontal(|ui| {
                if on {
                    ui.label(dim(trf(
                        "Label_Threshold",
                        &[&format!("{:.0}", s.noise_gate_db)],
                    )));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if running {
                            let (t, c) = if self.meters.gate_open {
                                (tr("Gate_Open"), GOOD)
                            } else {
                                (tr("Gate_Closed"), TEXT_DIM)
                            };
                            badge(ui, &t, c);
                        }
                    });
                } else {
                    ui.label(faint(tr("Gate_Off")));
                }
            });
        });
    }
}

fn parse_freq(s: &str) -> Option<f64> {
    let t = s.trim().to_lowercase().replace("hz", "");
    let t = t.trim();
    if let Some(k) = t.strip_suffix('k') {
        k.trim().parse::<f64>().ok().map(|v| v * 1000.0)
    } else {
        t.parse().ok()
    }
}

// ---- dialogs ------------------------------------------------------------------------

impl App {
    fn dialogs(&mut self, ctx: &egui::Context, snap: &Snapshot) {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        let mut keep = true;
        let modal = egui::Modal::new(egui::Id::new("dialog")).frame(
            Frame::new()
                .fill(SURFACE)
                .stroke(Stroke::new(1.0, BORDER))
                .corner_radius(CornerRadius::same(RADIUS))
                .inner_margin(Margin::same(20)),
        );
        let resp = modal.show(ctx, |ui| {
            ui.set_width(380.0);
            match &mut dialog {
                Dialog::SavePresetAs { name, error, focus } => {
                    ui.label(RichText::new(tr("Dialog_SavePreset")).font(bold_font(16.0)));
                    ui.label(faint(tr("Dialog_SavePresetHint")));
                    let r = ui.add(
                        TextEdit::singleline(name)
                            .hint_text(tr("Dialog_PresetName"))
                            .desired_width(f32::INFINITY),
                    );
                    if std::mem::take(focus) {
                        r.request_focus();
                    }
                    let exists = lock(&self.ctrl).preset_exists(name);
                    if exists {
                        ui.label(
                            RichText::new(trf("Msg_PresetOverwrite", &[&name.trim()]))
                                .color(WARN)
                                .font(font(12.0)),
                        );
                    }
                    if let Some(e) = error {
                        ui.label(RichText::new(e.as_str()).color(BAD));
                    }
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let (ok, cancel) =
                        dialog_buttons(ui, &tr(if exists { "Btn_Overwrite" } else { "Btn_Save" }));
                    if ok || enter {
                        match lock(&self.ctrl).save_preset_as(name) {
                            Ok(()) => keep = false,
                            Err(e) => *error = Some(e),
                        }
                    }
                    keep &= !cancel;
                }
                Dialog::RenamePreset {
                    old,
                    name,
                    error,
                    focus,
                } => {
                    ui.label(RichText::new(tr("Preset_Rename")).font(bold_font(16.0)));
                    let r = ui.add(TextEdit::singleline(name).desired_width(f32::INFINITY));
                    if std::mem::take(focus) {
                        r.request_focus();
                    }
                    if let Some(e) = error {
                        ui.label(RichText::new(e.as_str()).color(BAD));
                    }
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let (ok, cancel) = dialog_buttons(ui, &tr("Btn_Rename"));
                    if ok || enter {
                        match lock(&self.ctrl).rename_preset(old, name) {
                            Ok(()) => keep = false,
                            Err(e) => *error = Some(e),
                        }
                    }
                    keep &= !cancel;
                }
                Dialog::DeletePreset(name) => {
                    ui.label(RichText::new(trf("Msg_PresetDelete", &[name])).font(bold_font(15.0)));
                    ui.label(faint(tr("Msg_CannotUndo")));
                    let (ok, cancel) = danger_buttons(ui, &tr("Btn_Delete"));
                    if ok {
                        lock(&self.ctrl).delete_preset(name);
                        keep = false;
                    }
                    keep &= !cancel;
                }
                Dialog::SwitchPreset(target) => {
                    ui.label(
                        RichText::new(trf("Msg_UnsavedChanges", &[&snap.current_label]))
                            .font(bold_font(15.0)),
                    );
                    ui.label(faint(tr("Msg_UnsavedChangesHint")));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, tr("Btn_SaveAndSwitch")).clicked() {
                            if let Some(PresetId::User(n)) = &snap.current {
                                let r = lock(&self.ctrl).save_preset_as(n);
                                if self.report(r) {
                                    lock(&self.ctrl).apply_preset(target);
                                }
                            }
                            keep = false;
                        }
                        if ui
                            .button(RichText::new(tr("Btn_Discard")).color(BAD))
                            .clicked()
                        {
                            lock(&self.ctrl).apply_preset(target);
                            keep = false;
                        }
                        if ui.button(tr("Btn_Cancel")).clicked() {
                            keep = false;
                        }
                    });
                }
                Dialog::SaveProfile { name, error, focus } => {
                    ui.label(RichText::new(tr("Profile_SaveCurrent")).font(bold_font(16.0)));
                    ui.label(faint(format!(
                        "{}  {}\n{}  {}",
                        icon::MICROPHONE,
                        snap.devices
                            .input_name(&snap.settings.input_device_id)
                            .unwrap_or(&snap.settings.input_device_name),
                        icon::SPEAKER_HIGH,
                        snap.devices
                            .output_name(&snap.settings.output_device_id)
                            .unwrap_or(&snap.settings.output_device_name),
                    )));
                    let r = ui.add(
                        TextEdit::singleline(name)
                            .hint_text(tr("Dialog_ProfileName"))
                            .desired_width(f32::INFINITY),
                    );
                    if std::mem::take(focus) {
                        r.request_focus();
                    }
                    let exists = lock(&self.ctrl).profile_exists(name);
                    if exists {
                        ui.label(
                            RichText::new(trf("Msg_DeviceProfileOverwrite", &[&name.trim()]))
                                .color(WARN)
                                .font(font(12.0)),
                        );
                    }
                    if let Some(e) = error {
                        ui.label(RichText::new(e.as_str()).color(BAD));
                    }
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let (ok, cancel) =
                        dialog_buttons(ui, &tr(if exists { "Btn_Overwrite" } else { "Btn_Save" }));
                    if ok || enter {
                        match lock(&self.ctrl).save_profile_as(name) {
                            Ok(()) => keep = false,
                            Err(e) => *error = Some(e),
                        }
                    }
                    keep &= !cancel;
                }
                Dialog::DeleteProfile(name) => {
                    ui.label(
                        RichText::new(trf("Msg_DeviceProfileDelete", &[name]))
                            .font(bold_font(15.0)),
                    );
                    let (ok, cancel) = danger_buttons(ui, &tr("Btn_Delete"));
                    if ok {
                        lock(&self.ctrl).delete_profile(name);
                        keep = false;
                    }
                    keep &= !cancel;
                }
                Dialog::Settings => {
                    keep = self.settings_dialog(ui, snap);
                }
            }
        });
        if resp.should_close() {
            keep = false;
        }
        if keep {
            self.dialog = Some(dialog);
        }
    }

    fn settings_dialog(&mut self, ui: &mut Ui, snap: &Snapshot) -> bool {
        let s = &snap.settings;
        let mut keep = true;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{}  {}", icon::GEAR_SIX, tr("Settings_Title")))
                    .font(bold_font(17.0)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_button(ui, icon::X, &tr("Btn_Close")).clicked() {
                    keep = false;
                }
            });
        });
        ui.add_space(6.0);
        egui::Grid::new("settings")
            .num_columns(2)
            .spacing([16.0, 12.0])
            .show(ui, |ui| {
                ui.label(dim(tr("Settings_Language")));
                let current = i18n::LANGUAGES
                    .iter()
                    .find(|l| l.0 == s.language)
                    .map(|l| l.1)
                    .unwrap_or("English");
                egui::ComboBox::from_id_salt("lang")
                    .selected_text(current)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (code, native, _) in i18n::LANGUAGES {
                            if ui.selectable_label(s.language == *code, *native).clicked() {
                                lock(&self.ctrl).edit_settings(|x| x.language = code.to_string());
                            }
                        }
                    });
                ui.end_row();

                ui.label(dim(tr("Check_Startup")));
                let mut a = self.autostart;
                if toggle(ui, &mut a).changed() {
                    match platform::set_autostart(a) {
                        Ok(()) => self.autostart = platform::autostart_enabled(),
                        Err(e) => {
                            self.error = Some((trf("Msg_StartupFailed", &[&e]), Instant::now()))
                        }
                    }
                }
                ui.end_row();

                ui.label(dim(tr("Settings_CloseToTray")));
                let mut v = s.close_to_tray;
                if toggle(ui, &mut v).changed() {
                    lock(&self.ctrl).edit_settings(|x| x.close_to_tray = v);
                }
                ui.end_row();

                ui.label(dim(tr("Check_UnlockLimit")));
                let mut v = s.unlock_limit;
                if toggle(ui, &mut v)
                    .on_hover_text(tr("Tip_UnlockLimit"))
                    .changed()
                {
                    lock(&self.ctrl).edit_settings(|x| x.unlock_limit = v);
                }
                ui.end_row();

                ui.label(dim(tr("Settings_Ceiling")))
                    .on_hover_text(tr("Tip_Ceiling"));
                let mut c = s.limiter_ceiling_db;
                if ui
                    .add(
                        egui::Slider::new(&mut c, -12.0..=0.0)
                            .suffix(" dBFS")
                            .fixed_decimals(1),
                    )
                    .changed()
                {
                    lock(&self.ctrl).edit_settings(|x| x.limiter_ceiling_db = c);
                }
                ui.end_row();

                ui.label(dim(tr("Settings_CheckUpdates")));
                ui.horizontal(|ui| {
                    let mut v = s.check_updates;
                    if toggle(ui, &mut v).changed() {
                        lock(&self.ctrl).edit_settings(|x| x.check_updates = v);
                        if v {
                            let c = ui.ctx().clone();
                            update::check(
                                self.update.clone(),
                                Arc::new(move || c.request_repaint()),
                            );
                        }
                    }
                    let st = self.update.lock().unwrap().clone();
                    let text = match st {
                        UpdateState::Checking => tr("Msg_UpdateChecking"),
                        UpdateState::UpToDate => tr("Msg_UpToDate"),
                        UpdateState::Available(v) => trf("Msg_UpdateAvailable", &[&v]),
                        UpdateState::Failed => tr("Msg_UpdateError"),
                        UpdateState::Idle => String::new(),
                    };
                    ui.label(faint(text));
                });
                ui.end_row();
            });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui
                .button(format!(
                    "{}  {}",
                    icon::FOLDER_OPEN,
                    tr("Btn_OpenConfigFolder")
                ))
                .clicked()
            {
                platform::open_path(lock(&self.ctrl).store.dir());
            }
            if ui
                .button(format!("{}  GitHub", icon::GITHUB_LOGO))
                .clicked()
            {
                platform::open_url("https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster");
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button(
                        RichText::new(format!("{}  {}", icon::POWER, tr("Menu_Exit"))).color(BAD),
                    )
                    .clicked()
                {
                    self.allow_close = true;
                    exit_app(&self.ctrl);
                }
            });
        });
        ui.add_space(4.0);
        ui.label(faint(format!(
            "Easy MIC Booster v{} · MIT License",
            update::current_version()
        )));
        keep
    }

    fn error_toast(&mut self, ctx: &egui::Context) {
        let Some((msg, at)) = &self.error else { return };
        if at.elapsed() > Duration::from_secs(6) {
            self.error = None;
            return;
        }
        let msg = msg.clone();
        egui::Area::new(egui::Id::new("toast"))
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -24.0))
            .show(ctx, |ui| {
                Frame::new()
                    .fill(BAD.linear_multiply(0.2))
                    .stroke(Stroke::new(1.0, BAD))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("{}  {msg}", icon::WARNING_CIRCLE)).color(TEXT),
                        );
                    });
            });
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

fn dialog_buttons(ui: &mut Ui, ok: &str) -> (bool, bool) {
    ui.add_space(8.0);
    let mut r = (false, false);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        r.0 = primary_button(ui, ok).clicked();
        r.1 = ui.button(tr("Btn_Cancel")).clicked();
    });
    r
}

fn danger_buttons(ui: &mut Ui, ok: &str) -> (bool, bool) {
    ui.add_space(8.0);
    let mut r = (false, false);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        r.0 = ui
            .add(
                egui::Button::new(RichText::new(ok).color(Color32::WHITE))
                    .fill(BAD.linear_multiply(0.8)),
            )
            .clicked();
        r.1 = ui.button(tr("Btn_Cancel")).clicked();
    });
    r
}
