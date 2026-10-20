//! OS integration: single instance, autostart, window visibility, device
//! change notifications, opening folders and links.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;

#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use self::unix::*;

use std::path::Path;

pub fn exe_path() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok().or(Some(p)))
        .map(|p| {
            // canonicalize() adds the \\?\ prefix on Windows, which the Run key does not like.
            let s = p.to_string_lossy().to_string();
            std::path::PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s))
        })
}

pub fn open_path(path: &Path) {
    #[cfg(windows)]
    let cmd = ("explorer.exe", vec![path.as_os_str().to_owned()]);
    #[cfg(target_os = "macos")]
    let cmd = ("open", vec![path.as_os_str().to_owned()]);
    #[cfg(all(unix, not(target_os = "macos")))]
    let cmd = ("xdg-open", vec![path.as_os_str().to_owned()]);
    let _ = std::process::Command::new(cmd.0).args(cmd.1).spawn();
}

pub fn open_url(url: &str) {
    #[cfg(windows)]
    let _ = std::process::Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}
