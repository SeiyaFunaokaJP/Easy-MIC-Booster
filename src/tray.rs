//! Tray icon with quick controls. The menu is rebuilt from the controller state
//! whenever it changes, on the thread that owns the tray (the UI thread on
//! Windows/macOS, a GTK thread on Linux). Menu actions go straight to the
//! controller, so they work while the window is hidden.

use std::cell::RefCell;
use std::sync::OnceLock;

use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::audio::EngineStatus;
use crate::config::{EqMode, DEFAULT_DEVICE_ID};
use crate::controller::{builtin_presets, lock, PresetId, Shared};
use crate::i18n::{tr, trf};

/// Callbacks into the app that the tray needs.
pub struct TrayHooks {
    pub controller: Shared,
    pub show_window: Box<dyn Fn() + Send + Sync>,
    pub exit: Box<dyn Fn() + Send + Sync>,
}

static HOOKS: OnceLock<TrayHooks> = OnceLock::new();

#[derive(Clone)]
enum Action {
    Boost(bool),
    NoiseSuppression(bool),
    Gain(f32),
    GainStep(f32),
    Mode(EqMode),
    Preset(PresetId),
    Profile(String),
    Input(String),
    Output(String),
    Open,
    Exit,
}

struct TrayState {
    tray: TrayIcon,
    actions: Vec<Action>,
    built_version: u64,
    icon_on: Icon,
    icon_off: Icon,
    icon_is_on: Option<bool>,
}

thread_local! {
    static STATE: RefCell<Option<TrayState>> = const { RefCell::new(None) };
}

fn load_icons() -> (Icon, Icon) {
    let img = image::load_from_memory_with_format(
        include_bytes!("../assets/App.ico"),
        image::ImageFormat::Ico,
    )
    .map(|i| {
        i.resize(32, 32, image::imageops::FilterType::Lanczos3)
            .to_rgba8()
    })
    .unwrap_or_else(|_| image::RgbaImage::from_pixel(32, 32, image::Rgba([139, 92, 246, 255])));
    let (w, h) = img.dimensions();
    let on = img.clone().into_raw();
    // Dimmed, desaturated variant while boost is off or audio is not running.
    let off: Vec<u8> = on
        .chunks_exact(4)
        .flat_map(|p| {
            let l = (0.3 * p[0] as f32 + 0.59 * p[1] as f32 + 0.11 * p[2] as f32) as u8;
            [l, l, l, (p[3] as f32 * 0.55) as u8]
        })
        .collect();
    (
        Icon::from_rgba(on, w, h).expect("icon"),
        Icon::from_rgba(off, w, h).expect("icon"),
    )
}

/// Creates the tray on the current thread. Must be called once.
pub fn create(hooks: TrayHooks) {
    let _ = HOOKS.set(hooks);
    let (icon_on, icon_off) = load_icons();
    let tray = match TrayIconBuilder::new()
        .with_icon(icon_on.clone())
        .with_tooltip("Easy MIC Booster")
        .with_menu_on_left_click(false)
        .build()
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("tray icon unavailable: {e}");
            return;
        }
    };
    STATE.with(|s| {
        *s.borrow_mut() = Some(TrayState {
            tray,
            actions: Vec::new(),
            built_version: 0,
            icon_on,
            icon_off,
            icon_is_on: None,
        })
    });
    refresh();

    MenuEvent::set_event_handler(Some(|e: MenuEvent| {
        let action = STATE.with(|s| {
            let s = s.borrow();
            let idx: usize = e.id.0.strip_prefix('a')?.parse().ok()?;
            s.as_ref()?.actions.get(idx).cloned()
        });
        if let Some(a) = action {
            run(a);
        }
        refresh();
    }));
    TrayIconEvent::set_event_handler(Some(|e: TrayIconEvent| match e {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
        | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => {
            if let Some(h) = HOOKS.get() {
                (h.show_window)();
            }
        }
        // Make sure the menu is current right before it can be opened.
        TrayIconEvent::Enter { .. } | TrayIconEvent::Move { .. } => refresh(),
        _ => {}
    }));
}

fn run(action: Action) {
    let Some(h) = HOOKS.get() else { return };
    match action {
        Action::Open => (h.show_window)(),
        Action::Exit => (h.exit)(),
        a => {
            let mut c = lock(&h.controller);
            match a {
                Action::Boost(on) => c.set_boost(on),
                Action::NoiseSuppression(on) => c.edit_sound(|s| s.noise_suppression = on),
                Action::Gain(db) => c.edit_sound(|s| s.gain_db = db),
                Action::GainStep(d) => {
                    c.edit_sound(|s| s.gain_db = ((s.gain_db + d) / 3.0).round() * 3.0)
                }
                Action::Mode(m) => c.set_mode(m),
                Action::Preset(id) => c.apply_preset(&id),
                Action::Profile(n) => c.apply_profile(&n),
                Action::Input(id) => c.set_input(&id),
                Action::Output(id) => c.set_output(&id),
                Action::Open | Action::Exit => {}
            }
            // Tray changes are discrete: write them out right away.
            c.save_now();
        }
    }
}

/// Rebuilds the menu if the controller changed since the last build.
/// Must run on the tray's thread.
pub fn refresh() {
    let Some(h) = HOOKS.get() else { return };
    STATE.with(|cell| {
        let mut guard = cell.borrow_mut();
        let Some(st) = guard.as_mut() else { return };
        let c = lock(&h.controller);
        let status = c.status();
        let version =
            c.version + c.engine.version.load(std::sync::atomic::Ordering::Relaxed) * 1_000_000;
        if version == st.built_version {
            return;
        }
        st.built_version = version;

        let mut actions = Vec::new();
        let mut act = |a: Action| -> String {
            actions.push(a);
            format!("a{}", actions.len() - 1)
        };
        let s = &c.settings;
        let running = matches!(status, EngineStatus::Running { .. });
        let menu = Menu::new();

        let status_text = match &status {
            EngineStatus::Running { .. } if s.boost_enabled => {
                trf("Tray_StatusBoost", &[&format_db(s.sound.gain_db)])
            }
            EngineStatus::Running { .. } => tr("Status_Bypassed"),
            EngineStatus::MissingDevice { .. } => tr("Status_DeviceMissing"),
            EngineStatus::Error(_) => tr("Status_Error"),
            EngineStatus::Starting => tr("Status_Starting"),
            EngineStatus::Idle => tr("Status_ChooseOutput"),
        };
        let _ = menu.append(&MenuItem::new(
            format!("Easy MIC Booster — {status_text}"),
            false,
            None,
        ));
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&CheckMenuItem::with_id(
            act(Action::Boost(!s.boost_enabled)),
            tr("Tray_Boost"),
            true,
            s.boost_enabled,
            None,
        ));
        let ns = s.sound.noise_suppression;
        let _ = menu.append(&CheckMenuItem::with_id(
            act(Action::NoiseSuppression(!ns)),
            tr("Tray_NoiseSuppression"),
            true,
            ns,
            None,
        ));

        // Gain
        let gain = Submenu::new(trf("Tray_Gain", &[&format_db(s.sound.gain_db)]), true);
        let _ = gain.append(&MenuItem::with_id(
            act(Action::GainStep(3.0)),
            "+3 dB",
            true,
            None,
        ));
        let _ = gain.append(&MenuItem::with_id(
            act(Action::GainStep(-3.0)),
            "−3 dB",
            true,
            None,
        ));
        let _ = gain.append(&PredefinedMenuItem::separator());
        for db in [0.0f32, 6.0, 12.0, 18.0, 24.0, 30.0, 36.0] {
            if db <= s.max_gain_db() {
                let checked = (s.sound.gain_db - db).abs() < 0.05;
                let _ = gain.append(&CheckMenuItem::with_id(
                    act(Action::Gain(db)),
                    format_db(db),
                    true,
                    checked,
                    None,
                ));
            }
        }
        let _ = gain.append(&PredefinedMenuItem::separator());
        let eq_on = s.sound.mode == EqMode::Equalizer;
        let _ = gain.append(&CheckMenuItem::with_id(
            act(Action::Mode(if eq_on {
                EqMode::Simple
            } else {
                EqMode::Equalizer
            })),
            tr("Tray_UseEq"),
            !s.sound.bands.is_empty() || eq_on,
            eq_on,
            None,
        ));
        let _ = menu.append(&gain);

        // Presets
        let current = c.current_preset();
        let modified = c.preset_modified();
        let presets = Submenu::new(
            match &current {
                Some(id) => {
                    let mark = if modified { " *" } else { "" };
                    trf(
                        "Tray_PresetCurrent",
                        &[&format!("{}{mark}", c.preset_label(id))],
                    )
                }
                None => tr("Tray_Presets"),
            },
            true,
        );
        for p in &c.presets {
            let id = PresetId::User(p.name.clone());
            let checked = current.as_ref() == Some(&id);
            let _ = presets.append(&CheckMenuItem::with_id(
                act(Action::Preset(id)),
                &p.name,
                true,
                checked,
                None,
            ));
        }
        if !c.presets.is_empty() {
            let _ = presets.append(&PredefinedMenuItem::separator());
        }
        for (bid, key, _) in builtin_presets() {
            let id = PresetId::Builtin(bid);
            let checked = current.as_ref() == Some(&id);
            let _ = presets.append(&CheckMenuItem::with_id(
                act(Action::Preset(id)),
                tr(key),
                true,
                checked,
                None,
            ));
        }
        let _ = menu.append(&presets);

        let _ = menu.append(&PredefinedMenuItem::separator());

        // Devices
        let devices = c.devices();
        if !c.profiles.is_empty() {
            let active = c.active_profile().map(|p| p.name.clone());
            let sub = Submenu::new(tr("Tray_DeviceProfiles"), true);
            for p in &c.profiles {
                let checked = active.as_deref() == Some(p.name.as_str());
                let _ = sub.append(&CheckMenuItem::with_id(
                    act(Action::Profile(p.name.clone())),
                    &p.name,
                    true,
                    checked,
                    None,
                ));
            }
            let _ = menu.append(&sub);
        }
        let device_menu = |title: String,
                           list: &[crate::audio::DeviceInfo],
                           default: Option<&str>,
                           current: &str,
                           input: bool,
                           act: &mut dyn FnMut(Action) -> String| {
            let sub = Submenu::new(title, true);
            let default_name = default
                .and_then(|d| list.iter().find(|x| x.id == d))
                .map(|x| x.name.as_str());
            let label = match default_name {
                Some(n) => trf("Device_Default", &[&n]),
                None => tr("Device_Default_None"),
            };
            let mk = |id: &str| {
                if input {
                    Action::Input(id.into())
                } else {
                    Action::Output(id.into())
                }
            };
            let _ = sub.append(&CheckMenuItem::with_id(
                act(mk(DEFAULT_DEVICE_ID)),
                label,
                true,
                current == DEFAULT_DEVICE_ID,
                None,
            ));
            let _ = sub.append(&PredefinedMenuItem::separator());
            for d in list {
                let _ = sub.append(&CheckMenuItem::with_id(
                    act(mk(&d.id)),
                    &d.name,
                    true,
                    current == d.id,
                    None,
                ));
            }
            sub
        };
        let input_sub = device_menu(
            tr("Tray_Input"),
            &devices.inputs,
            devices.default_input.as_deref(),
            &s.input_device_id,
            true,
            &mut act,
        );
        let output_sub = device_menu(
            tr("Tray_Output"),
            &devices.outputs,
            devices.default_output.as_deref(),
            &s.output_device_id,
            false,
            &mut act,
        );
        let _ = menu.append(&input_sub);
        let _ = menu.append(&output_sub);

        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&MenuItem::with_id(
            act(Action::Open),
            tr("Menu_Open"),
            true,
            None,
        ));
        let _ = menu.append(&MenuItem::with_id(
            act(Action::Exit),
            tr("Menu_Exit"),
            true,
            None,
        ));

        let tooltip = format!(
            "Easy MIC Booster — {status_text}{}",
            if ns { " · AI NS" } else { "" }
        );
        let on = running && s.boost_enabled;
        drop(c);

        st.actions = actions;
        st.tray.set_menu(Some(Box::new(menu)));
        let _ = st.tray.set_tooltip(Some(truncate(&tooltip, 120)));
        if st.icon_is_on != Some(on) {
            st.icon_is_on = Some(on);
            let _ = st.tray.set_icon(Some(if on {
                st.icon_on.clone()
            } else {
                st.icon_off.clone()
            }));
        }
    });
}

fn truncate(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

pub fn format_db(db: f32) -> String {
    if db.abs() < 0.05 {
        "0 dB".into()
    } else {
        format!("{:+.1} dB", db)
    }
}

/// Linux: tray-icon needs a GTK main loop, which runs on its own thread.
#[cfg(target_os = "linux")]
pub fn spawn_linux(hooks: TrayHooks) {
    std::thread::Builder::new()
        .name("tray-gtk".into())
        .spawn(move || {
            if gtk::init().is_err() {
                eprintln!("GTK unavailable; running without a tray icon");
                return;
            }
            create(hooks);
            gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), || {
                refresh();
                gtk::glib::ControlFlow::Continue
            });
            gtk::main();
        })
        .ok();
}
