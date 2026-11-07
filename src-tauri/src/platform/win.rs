use super::Snapshot;
use std::{mem::zeroed, ptr::null};
use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT};
use windows_sys::Win32::Graphics::Dwm::DwmGetWindowAttribute;
use windows_sys::Win32::UI::Shell::SHQueryUserNotificationState;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

struct Ctx {
    own: u32,
    rects: Vec<[f64; 4]>,
}

fn to_rect(r: &RECT) -> [f64; 4] {
    [r.left as f64, r.top as f64, (r.right - r.left) as f64, (r.bottom - r.top) as f64]
}

unsafe fn visible_frame(hwnd: HWND) -> Option<RECT> {
    let mut r: RECT = zeroed();
    // 9 = DWMWA_EXTENDED_FRAME_BOUNDS (without the invisible resize border)
    if DwmGetWindowAttribute(hwnd, 9 as _, &mut r as *mut RECT as *mut _, std::mem::size_of::<RECT>() as u32) == 0 {
        return Some(r);
    }
    if GetWindowRect(hwnd, &mut r) != 0 {
        Some(r)
    } else {
        None
    }
}

unsafe extern "system" fn cb(hwnd: HWND, lp: LPARAM) -> i32 {
    let ctx = &mut *(lp as *mut Ctx);
    if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
        return 1;
    }
    let ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    if ex & WS_EX_TOOLWINDOW != 0 || ex & WS_EX_NOACTIVATE != 0 {
        return 1;
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == ctx.own {
        return 1;
    }
    let mut cloaked = 0u32; // 14 = DWMWA_CLOAKED (windows on other virtual desktops, UWP ghosts)
    DwmGetWindowAttribute(hwnd, 14 as _, &mut cloaked as *mut u32 as *mut _, 4);
    if cloaked != 0 {
        return 1;
    }
    if let Some(r) = visible_frame(hwnd) {
        let b = to_rect(&r);
        if b[2] >= 80.0 && b[3] >= 40.0 {
            ctx.rects.push(b);
        }
    }
    1
}

pub fn snapshot(own: u32) -> Snapshot {
    unsafe {
        let mut ctx = Ctx { own, rects: Vec::new() };
        EnumWindows(Some(cb), &mut ctx as *mut Ctx as LPARAM); // already in z-order, top first

        let class: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
        let tb = FindWindowW(class.as_ptr(), null());
        if tb as usize != 0 && IsWindowVisible(tb) != 0 {
            let mut r: RECT = zeroed();
            if GetWindowRect(tb, &mut r) != 0 {
                ctx.rects.push(to_rect(&r));
            }
        }

        // 2 = busy (fullscreen app), 3 = D3D fullscreen, 4 = presentation mode
        let mut state: i32 = 0;
        let fs = SHQueryUserNotificationState(&mut state) == 0 && matches!(state, 2 | 3 | 4);
        Snapshot { rects: ctx.rects, fullscreen: fs }
    }
}
