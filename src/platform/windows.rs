use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Arc;
use std::thread;

use windows::core::{implement, w, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND, PROPERTYKEY,
};
use windows::Win32::Media::Audio::{
    EDataFlow, ERole, IMMDeviceEnumerator, IMMNotificationClient, IMMNotificationClient_Impl,
    MMDeviceEnumerator, DEVICE_STATE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForSingleObject, EVENT_MODIFY_STATE,
    INFINITE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow, ASFW_ANY,
    SW_HIDE, SW_RESTORE, SW_SHOW,
};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

/// Same name as v1.x, so the old and new versions never route at the same time,
/// and the installer's AppMutex check keeps working.
#[cfg(not(debug_assertions))]
const MUTEX_NAME: PCWSTR = w!("Global\\EasyMICBooster_UniqueInstance_Mutex");
#[cfg(not(debug_assertions))]
const SHOW_EVENT: PCWSTR = w!("Local\\EasyMICBooster_ShowWindow");
// Debug builds can run next to an installed release.
#[cfg(debug_assertions)]
const MUTEX_NAME: PCWSTR = w!("Local\\EasyMICBooster_Dev_Mutex");
#[cfg(debug_assertions)]
const SHOW_EVENT: PCWSTR = w!("Local\\EasyMICBooster_Dev_ShowWindow");

pub struct InstanceGuard {
    _mutex: HANDLE,
}

// The handles are only closed at process exit.
unsafe impl Send for InstanceGuard {}

/// Returns `None` if another instance runs (after asking it to show its window).
pub fn acquire_single_instance(on_show: impl Fn() + Send + 'static) -> Option<InstanceGuard> {
    unsafe {
        let mutex = CreateMutexW(None, true, MUTEX_NAME).ok()?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            if let Ok(ev) = OpenEventW(EVENT_MODIFY_STATE, false, SHOW_EVENT) {
                let _ = AllowSetForegroundWindow(ASFW_ANY);
                let _ = SetEvent(ev);
                let _ = CloseHandle(ev);
            }
            let _ = CloseHandle(mutex);
            return None;
        }
        if let Ok(ev) = CreateEventW(None, false, false, SHOW_EVENT) {
            let ev = ev.0 as isize;
            thread::Builder::new()
                .name("instance-signal".into())
                .spawn(move || loop {
                    WaitForSingleObject(HANDLE(ev as *mut _), INFINITE);
                    on_show();
                })
                .ok();
        }
        Some(InstanceGuard { _mutex: mutex })
    }
}

static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);

pub fn set_main_window(hwnd: isize) {
    MAIN_HWND.store(hwnd, Ordering::Relaxed);
}

fn hwnd() -> Option<HWND> {
    match MAIN_HWND.load(Ordering::Relaxed) {
        0 => None,
        h => Some(HWND(h as *mut _)),
    }
}

/// Native show/hide: egui stops running frames while its window is hidden, so a
/// viewport command could never bring it back.
pub fn show_main_window() -> bool {
    let Some(h) = hwnd() else { return false };
    unsafe {
        let _ = ShowWindow(h, SW_SHOW);
        if IsIconic(h).as_bool() {
            let _ = ShowWindow(h, SW_RESTORE);
        }
        let _ = SetForegroundWindow(h);
    }
    true
}

pub fn hide_main_window() -> bool {
    let Some(h) = hwnd() else { return false };
    unsafe {
        let _ = ShowWindow(h, SW_HIDE);
    }
    true
}

pub fn main_window_visible() -> bool {
    hwnd().is_some_and(|h| unsafe { IsWindowVisible(h).as_bool() && !IsIconic(h).as_bool() })
}

pub const SUPPORTS_HIDE: bool = true;

// ---- autostart ---------------------------------------------------------------

const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const APPROVED_KEY: &str =
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const VALUE: &str = "EasyMICBooster";

fn run_command() -> Option<String> {
    Some(format!("\"{}\" --minimized", super::exe_path()?.display()))
}

pub fn autostart_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(run) = hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ) else {
        return false;
    };
    if run.get_value::<String, _>(VALUE).is_err() {
        return false;
    }
    // StartupApproved first byte: 0x02 enabled, 0x03 disabled in Task Manager.
    if let Ok(approved) = hkcu.open_subkey_with_flags(APPROVED_KEY, KEY_READ) {
        if let Ok(v) = approved.get_raw_value(VALUE) {
            if v.bytes.first().is_some_and(|b| b & 1 != 0) {
                return false;
            }
        }
    }
    true
}

pub fn set_autostart(enable: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = hkcu.create_subkey(RUN_KEY).map_err(|e| e.to_string())?;
    if enable {
        let cmd = run_command().ok_or_else(|| crate::i18n::tr("Msg_StartupPathFailed"))?;
        run.set_value(VALUE, &cmd).map_err(|e| e.to_string())?;
        if let Ok((approved, _)) = hkcu.create_subkey(APPROVED_KEY) {
            let mut bytes = vec![0u8; 12];
            bytes[0] = 2;
            let _ = approved.set_raw_value(
                VALUE,
                &winreg::RegValue {
                    bytes,
                    vtype: winreg::enums::RegType::REG_BINARY,
                },
            );
        }
    } else {
        let _ = run.delete_value(VALUE);
    }
    Ok(())
}

/// Keeps the Run entry pointing at this exe after it was moved or updated.
pub fn refresh_autostart_path() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(run) = hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ | KEY_WRITE) else {
        return;
    };
    let (Ok(old), Some(cmd)) = (run.get_value::<String, _>(VALUE), run_command()) else {
        return;
    };
    if old != cmd {
        let _ = run.set_value(VALUE, &cmd);
    }
}

// ---- device notifications ------------------------------------------------------

#[implement(IMMNotificationClient)]
struct Notifier(Arc<AtomicBool>);

impl IMMNotificationClient_Impl for Notifier_Impl {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: DEVICE_STATE) -> windows::core::Result<()> {
        self.0.store(true, Ordering::Relaxed);
        Ok(())
    }
    fn OnDeviceAdded(&self, _: &PCWSTR) -> windows::core::Result<()> {
        self.0.store(true, Ordering::Relaxed);
        Ok(())
    }
    fn OnDeviceRemoved(&self, _: &PCWSTR) -> windows::core::Result<()> {
        self.0.store(true, Ordering::Relaxed);
        Ok(())
    }
    fn OnDefaultDeviceChanged(
        &self,
        _: EDataFlow,
        _: ERole,
        _: &PCWSTR,
    ) -> windows::core::Result<()> {
        self.0.store(true, Ordering::Relaxed);
        Ok(())
    }
    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> windows::core::Result<()> {
        Ok(())
    }
}

/// Sets `flag` whenever an audio endpoint appears, disappears or the default changes.
pub fn watch_devices(flag: Arc<AtomicBool>) {
    thread::Builder::new()
        .name("device-notifications".into())
        .spawn(move || unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let Ok(enumerator) =
                CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
            else {
                return;
            };
            let client: IMMNotificationClient = Notifier(flag).into();
            if enumerator
                .RegisterEndpointNotificationCallback(&client)
                .is_ok()
            {
                // Both must stay alive for the callbacks to keep coming.
                loop {
                    thread::park();
                }
            }
        })
        .ok();
}
