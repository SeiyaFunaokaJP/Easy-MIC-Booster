use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

pub struct InstanceGuard {
    path: PathBuf,
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn socket_path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let user = std::env::var("USER").unwrap_or_default();
    dir.join(format!("easymicbooster-{user}.sock"))
}

/// Returns `None` if another instance runs (after asking it to show its window).
pub fn acquire_single_instance(on_show: impl Fn() + Send + 'static) -> Option<InstanceGuard> {
    let path = socket_path();
    if let Ok(mut s) = UnixStream::connect(&path) {
        let _ = s.write_all(b"show");
        return None;
    }
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).ok()?;
    thread::Builder::new()
        .name("instance-signal".into())
        .spawn(move || {
            for _ in listener.incoming().flatten() {
                on_show();
            }
        })
        .ok();
    Some(InstanceGuard { path })
}

pub fn set_main_window(_: isize) {}

/// egui cannot bring back a hidden window here, so the app minimizes instead.
pub const SUPPORTS_HIDE: bool = false;

pub fn show_main_window() -> bool {
    false
}

pub fn hide_main_window() -> bool {
    false
}

pub fn main_window_visible() -> bool {
    true
}

/// Devices are polled by the audio supervisor on these platforms.
pub fn watch_devices(_flag: Arc<AtomicBool>) {}

// ---- autostart ---------------------------------------------------------------

#[cfg(not(target_os = "macos"))]
fn autostart_file() -> Option<PathBuf> {
    Some(
        dirs::config_dir()?
            .join("autostart")
            .join("EasyMICBooster.desktop"),
    )
}

#[cfg(target_os = "macos")]
fn autostart_file() -> Option<PathBuf> {
    Some(
        dirs::home_dir()?
            .join("Library/LaunchAgents/io.github.seiyafunaokajp.easymicbooster.plist"),
    )
}

fn autostart_contents() -> Option<String> {
    let exe = super::exe_path()?;
    let exe = exe.display();
    #[cfg(not(target_os = "macos"))]
    let text = format!(
        "[Desktop Entry]\nType=Application\nName=Easy MIC Booster\nExec=\"{exe}\" --minimized\n\
         X-GNOME-Autostart-enabled=true\nTerminal=false\n"
    );
    #[cfg(target_os = "macos")]
    let text = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict>\
         <key>Label</key><string>io.github.seiyafunaokajp.easymicbooster</string>\
         <key>ProgramArguments</key><array><string>{exe}</string><string>--minimized</string></array>\
         <key>RunAtLoad</key><true/></dict></plist>\n"
    );
    Some(text)
}

pub fn autostart_enabled() -> bool {
    autostart_file().is_some_and(|p| p.exists())
}

pub fn set_autostart(enable: bool) -> Result<(), String> {
    let path = autostart_file().ok_or("no config directory")?;
    if enable {
        let text = autostart_contents().ok_or_else(|| crate::i18n::tr("Msg_StartupPathFailed"))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, text).map_err(|e| e.to_string())
    } else {
        let _ = std::fs::remove_file(&path);
        Ok(())
    }
}

pub fn refresh_autostart_path() {
    if autostart_enabled() {
        if let (Some(path), Some(text)) = (autostart_file(), autostart_contents()) {
            if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
                let _ = std::fs::write(path, text);
            }
        }
    }
}
