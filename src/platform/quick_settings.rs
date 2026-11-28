//! Windows 11's Quick Settings (the panel that the network and volume buttons
//! on the taskbar open): when they open and where they are, so the quick
//! controls can stand above them. Ported from BlueCaster.
//!
//! No API covers them. Their window has a class of its own, stays hidden
//! while they are closed and is the foreground window as they open (the input
//! method can take that over while they stay open), but it is a column as tall
//! as the screen and mostly transparent. Where the panels are comes from UI
//! Automation.

use std::cell::Cell;
use std::mem::ManuallyDrop;
use std::sync::OnceLock;

use windows::core::{BOOL, BSTR, HSTRING};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::System::Variant::{VariantClear, VARIANT, VT_BSTR};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationCondition, SetWinEventHook, TreeScope_Descendants,
    UIA_AutomationIdPropertyId, HWINEVENTHOOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, GetClassNameW, GetCursorPos, GetWindowLongPtrW, GetWindowRect,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetWindowLongPtrW, ShowWindow,
    EVENT_SYSTEM_FOREGROUND, GWL_EXSTYLE, SW_SHOWNOACTIVATE, WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS, WS_EX_NOACTIVATE, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};

/// The window's class, the same in every language.
const WINDOW_CLASS: &str = "ControlCenterWindow";
/// The panels (automation ids, the same in every language): the media card,
/// shown while something plays, and the one with the tiles and the volume.
const PANELS: [&str; 2] = ["MediaTransportControls", "ControlCenterRegion"];

static ON_OPEN: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/// Quick Settings as described here exist from Windows 11 (build 22000) on.
pub fn supported() -> bool {
    static SUPPORTED: OnceLock<bool> = OnceLock::new();
    *SUPPORTED.get_or_init(|| {
        let key = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
            .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
        key.and_then(|k| k.get_value::<String, _>("CurrentBuildNumber"))
            .ok()
            .and_then(|b| b.parse::<u32>().ok())
            .is_some_and(|b| b >= 22000)
    })
}

fn is_quick_settings(hwnd: HWND) -> bool {
    let mut name = [0u16; 32];
    // SAFETY: a buffer of its length; an invalid window gives 0.
    let len = unsafe { GetClassNameW(hwnd, &mut name) } as usize;
    String::from_utf16_lossy(&name[..len]) == WINDOW_CLASS
}

/// Their window while they are open, whichever window has the foreground.
fn open_window() -> Option<HWND> {
    let class = HSTRING::from(WINDOW_CLASS);
    let mut after = None;
    // SAFETY: the windows are only looked at; a window that is gone is not
    // visible, and the search then ends with an error.
    unsafe {
        while let Ok(hwnd) = FindWindowExW(None, after, &class, None) {
            if IsWindowVisible(hwnd).as_bool() {
                return Some(hwnd);
            }
            after = Some(hwnd);
        }
    }
    None
}

/// Whether Quick Settings are open (their window shows).
pub fn is_open() -> bool {
    open_window().is_some()
}

unsafe extern "system" fn on_foreground(
    _: HWINEVENTHOOK,
    _: u32,
    hwnd: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    if is_quick_settings(hwnd) {
        if let Some(on_open) = ON_OPEN.get() {
            on_open();
        }
    }
}

/// Calls `on_open` each time Quick Settings open, on this thread, which must
/// run a message loop. Once per process; it lasts until the process ends.
pub fn watch_opened(on_open: impl Fn() + Send + Sync + 'static) {
    if ON_OPEN.set(Box::new(on_open)).is_err() {
        return;
    }
    // SAFETY: the callback is a function of this program, called on this
    // thread's message loop (OUTOFCONTEXT); it needs no module.
    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(on_foreground),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    };
    if hook.is_invalid() {
        eprintln!("quick settings: no foreground hook");
    }
}

/// Joins this thread to the process's multithreaded COM apartment, for a
/// `Reader`. Lasts as long as the thread.
pub fn init_com() -> windows::core::Result<()> {
    // SAFETY: no reserved argument; each successful call is balanced by the
    // thread ending.
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok() }
}

/// Where the panels are (physical pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Panels {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Reads the panels of the Quick Settings open now. Keeps its COM objects, so
/// one thread can read often. That thread needs COM initialized.
pub struct Reader {
    automation: IUIAutomation,
    panels: IUIAutomationCondition,
    last: Cell<Option<Panels>>,
}

impl Reader {
    pub fn new() -> windows::core::Result<Self> {
        // SAFETY: the class and interface belong together.
        let automation: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)? };
        let id = |automation: &IUIAutomation,
                  id: &str|
         -> windows::core::Result<IUIAutomationCondition> {
            let mut value = VARIANT::default();
            // SAFETY: tags the VARIANT as the BSTR it now holds, which
            // VariantClear frees.
            unsafe {
                let v = &mut *value.Anonymous.Anonymous;
                v.vt = VT_BSTR;
                v.Anonymous.bstrVal = ManuallyDrop::new(BSTR::from(id));
                let condition =
                    automation.CreatePropertyCondition(UIA_AutomationIdPropertyId, &value);
                let _ = VariantClear(&mut value);
                condition
            }
        };
        let (a, b) = (id(&automation, PANELS[0])?, id(&automation, PANELS[1])?);
        // SAFETY: two valid conditions.
        let panels = unsafe { automation.CreateOrCondition(&a, &b)? };
        Ok(Self {
            automation,
            panels,
            last: Cell::new(None),
        })
    }

    /// The panels; None once Quick Settings are closed. If UI Automation does
    /// not find them, the last ones it did: as Quick Settings close, the panels
    /// go a moment before the window. Failing that, the whole window.
    pub fn read(&self) -> Option<Panels> {
        let hwnd = open_window()?;
        if let Some(panels) = self.bounds(hwnd) {
            self.last.set(Some(panels));
        }
        self.last.get().or_else(|| {
            let mut r = RECT::default();
            // SAFETY: a RECT to fill in. The first read can take seconds, so
            // the window is looked at again.
            unsafe {
                if !IsWindowVisible(hwnd).as_bool() {
                    return None;
                }
                GetWindowRect(hwnd, &mut r).ok()?;
            }
            Some(Panels {
                left: r.left,
                top: r.top,
                right: r.right,
                bottom: r.bottom,
            })
        })
    }

    fn bounds(&self, hwnd: HWND) -> Option<Panels> {
        // SAFETY: COM calls on objects this struct holds or that they return.
        let rects = unsafe {
            let found = self
                .automation
                .ElementFromHandle(hwnd)
                .ok()?
                .FindAll(TreeScope_Descendants, &self.panels)
                .ok()?;
            let mut rects = Vec::new();
            for i in 0..found.Length().ok()? {
                rects.push(found.GetElement(i).ok()?.CurrentBoundingRectangle().ok()?);
            }
            rects
        };
        union(&rects)
    }
}

/// Shows `hwnd` without activating it: activating any window closes Quick
/// Settings.
pub fn show_inactive(hwnd: isize) {
    // SAFETY: only changes whether the window shows; an invalid handle fails.
    let _ = unsafe { ShowWindow(HWND(hwnd as *mut _), SW_SHOWNOACTIVATE) };
}

/// Clicks on a WS_EX_NOACTIVATE window do not activate it, so Quick Settings
/// stay open while the quick controls above them are used.
pub fn set_no_activate(hwnd: isize, on: bool) {
    // SAFETY: reads and writes the window's extended style; an invalid handle
    // makes both calls fail.
    unsafe {
        let h = HWND(hwnd as *mut _);
        // The extended style fits 32 bits. GetWindowLongPtrW returns isize on
        // 64-bit Windows and i32 on 32-bit Windows.
        let style = GetWindowLongPtrW(h, GWL_EXSTYLE) as u32;
        let flag = WS_EX_NOACTIVATE.0;
        let next = if on { style | flag } else { style & !flag };
        if next != style {
            SetWindowLongPtrW(h, GWL_EXSTYLE, next as _);
        }
    }
}

/// Whether the pointer is over `hwnd`.
pub fn pointer_over(hwnd: isize) -> bool {
    let (mut at, mut r) = (POINT::default(), RECT::default());
    // SAFETY: a POINT and a RECT to fill in; an invalid handle fails.
    let read = unsafe {
        GetCursorPos(&mut at).is_ok() && GetWindowRect(HWND(hwnd as *mut _), &mut r).is_ok()
    };
    read && (r.left..r.right).contains(&at.x) && (r.top..r.bottom).contains(&at.y)
}

/// `panels` extended up to the cards other apps keep on top of them, so the
/// quick controls stand above those as well.
pub fn with_other_cards(panels: Panels, gap: i32) -> Panels {
    let mut cards = Vec::new();
    // SAFETY: the Vec outlives the call, which runs the callback on this thread.
    let _ = unsafe {
        EnumWindows(
            Some(collect_card),
            LPARAM(&mut cards as *mut Vec<RECT> as isize),
        )
    };
    Panels {
        top: top_of_cards(&panels, &cards, gap),
        ..panels
    }
}

unsafe extern "system" fn collect_card(hwnd: HWND, cards: LPARAM) -> BOOL {
    // SAFETY: with_other_cards passes its Vec, alive for the whole enumeration.
    let cards = unsafe { &mut *(cards.0 as *mut Vec<RECT>) };
    if let Some(r) = card_rect(hwnd) {
        cards.push(r);
    }
    true.into()
}

/// Where a window is, if it can be another app's card: visible, on top of
/// everything, and not passing clicks through as an overlay does.
fn card_rect(hwnd: HWND) -> Option<RECT> {
    let mut process = 0;
    // SAFETY: the windows are only looked at; each call takes a buffer of its
    // size, and a window that is gone fails them.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        if style & WS_EX_TOPMOST.0 == 0 || style & WS_EX_TRANSPARENT.0 != 0 {
            return None;
        }
        if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() || is_quick_settings(hwnd) {
            return None;
        }
        GetWindowThreadProcessId(hwnd, Some(&mut process));
        if process == GetCurrentProcessId() {
            return None;
        }
        let mut cloaked = 0u32;
        let size = size_of::<u32>() as u32;
        let read =
            DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, (&mut cloaked as *mut u32).cast(), size);
        if read.is_ok() && cloaked != 0 {
            return None;
        }
        // Without the shadow, where Windows has one.
        let mut r = RECT::default();
        let size = size_of::<RECT>() as u32;
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            (&mut r as *mut RECT).cast(),
            size,
        )
        .or_else(|_| GetWindowRect(hwnd, &mut r))
        .ok()?;
        Some(r)
    }
}

/// The top of the panels, or of the cards on them: a card lies over most of
/// the panels' width, with its bottom within `gap` of the top so far.
fn top_of_cards(panels: &Panels, cards: &[RECT], gap: i32) -> i32 {
    let width = panels.right - panels.left;
    let mut top = panels.top;
    while let Some(next) = cards
        .iter()
        .filter(|c| c.right.min(panels.right) - c.left.max(panels.left) > width / 2)
        .filter(|c| c.top < top && (c.bottom - top).abs() <= gap)
        .map(|c| c.top)
        .min()
    {
        top = next;
    }
    top
}

/// The smallest rectangle around the non-empty `rects`.
fn union(rects: &[RECT]) -> Option<Panels> {
    rects
        .iter()
        .filter(|r| r.right > r.left && r.bottom > r.top)
        .fold(None, |all, r| {
            Some(match all {
                None => Panels {
                    left: r.left,
                    top: r.top,
                    right: r.right,
                    bottom: r.bottom,
                },
                Some(a) => Panels {
                    left: a.left.min(r.left),
                    top: a.top.min(r.top),
                    right: a.right.max(r.right),
                    bottom: a.bottom.max(r.bottom),
                },
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_on_the_panels_raise_their_top() {
        let panels = Panels {
            left: 1000,
            top: 700,
            right: 1360,
            bottom: 1040,
        };
        let rect = |left, top, right, bottom| RECT {
            left,
            top,
            right,
            bottom,
        };
        assert_eq!(top_of_cards(&panels, &[], 12), 700);
        // A card on the panels, and one on that card.
        let card = rect(1000, 400, 1360, 690);
        assert_eq!(top_of_cards(&panels, &[card], 12), 400);
        assert_eq!(
            top_of_cards(&panels, &[rect(1010, 200, 1350, 395), card], 12),
            200
        );
        // A little over them is still on them.
        assert_eq!(
            top_of_cards(&panels, &[rect(1000, 400, 1360, 705)], 12),
            400
        );
        // Elsewhere on the screen, or a window as tall as the screen.
        let others = [
            rect(100, 400, 460, 690),
            rect(1000, 100, 1360, 500),
            rect(0, 0, 1366, 1040),
        ];
        assert_eq!(top_of_cards(&panels, &others, 12), 700);
    }

    #[test]
    fn union_takes_both_panels_and_skips_empty_ones() {
        // As measured: the media card above the tiles.
        let media = RECT {
            left: 3282,
            top: 1281,
            right: 3822,
            bottom: 1548,
        };
        let tiles = RECT {
            left: 3282,
            top: 1566,
            right: 3822,
            bottom: 2070,
        };
        let empty = RECT::default();
        assert_eq!(
            union(&[tiles, empty, media]),
            Some(Panels {
                left: 3282,
                top: 1281,
                right: 3822,
                bottom: 2070
            })
        );
        assert_eq!(
            union(&[tiles]),
            Some(Panels {
                left: 3282,
                top: 1566,
                right: 3822,
                bottom: 2070
            })
        );
        assert_eq!(union(&[empty]), None);
    }
}
