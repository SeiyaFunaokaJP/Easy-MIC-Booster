#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod config;
mod controller;
mod dsp;
mod i18n;
mod platform;
mod tray;
mod ui;
mod update;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};

use eframe::egui;

fn main() -> eframe::Result {
    let start_hidden = std::env::args().any(|a| a == "--minimized");

    let Some(_instance) = platform::acquire_single_instance(app::show_window) else {
        return Ok(());
    };

    // `--config-dir <path>`: keep settings somewhere else (portable use, testing).
    let args: Vec<String> = std::env::args().collect();
    let config_dir = args
        .iter()
        .position(|a| a == "--config-dir")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from);
    let store = match config_dir {
        Some(dir) => config::ConfigStore::at(dir),
        None => config::ConfigStore::open(),
    };
    install_panic_log(store.dir().join("crash.log"));

    // The egui context only exists once the window does; until then changes just
    // wait for the first frame.
    static CTX: OnceLock<egui::Context> = OnceLock::new();
    let notify: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {
        if let Some(c) = CTX.get() {
            c.request_repaint();
        }
    });
    let devices_dirty = Arc::new(AtomicBool::new(false));
    platform::watch_devices(devices_dirty.clone());
    let ctrl = controller::Controller::new(store, notify, devices_dirty);

    let hooks = || {
        let c = ctrl.clone();
        tray::TrayHooks {
            controller: ctrl.clone(),
            show_window: Box::new(app::show_window),
            exit: Box::new(move || app::exit_app(&c)),
        }
    };
    #[cfg(target_os = "linux")]
    tray::spawn_linux(hooks());

    let icon = image::load_from_memory_with_format(
        include_bytes!("../assets/App.ico"),
        image::ImageFormat::Ico,
    )
    .ok()
    .map(|i| {
        let i = i
            .resize(256, 256, image::imageops::FilterType::Lanczos3)
            .to_rgba8();
        let (width, height) = i.dimensions();
        egui::IconData {
            rgba: i.into_raw(),
            width,
            height,
        }
    });
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Easy MIC Booster")
        .with_app_id("easy-mic-booster")
        .with_inner_size([1080.0, 760.0])
        .with_min_inner_size([860.0, 600.0])
        // Only Windows can bring a window back from hidden; elsewhere start minimized.
        .with_visible(!(start_hidden && platform::SUPPORTS_HIDE));
    if let Some(icon) = icon {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    let ctrl_for_app = ctrl.clone();
    let result = eframe::run_native(
        "Easy MIC Booster",
        options,
        Box::new(move |cc| {
            let _ = CTX.set(cc.egui_ctx.clone());
            #[cfg(not(target_os = "linux"))]
            tray::create(hooks());
            Ok(Box::new(app::App::new(cc, ctrl_for_app, start_hidden)))
        }),
    );
    controller::lock(&ctrl).shutdown();
    result
}

fn install_panic_log(path: std::path::PathBuf) {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = std::fs::write(
            &path,
            format!("Easy MIC Booster v{}\n{info}\n", env!("CARGO_PKG_VERSION")),
        );
        default(info);
    }));
}
