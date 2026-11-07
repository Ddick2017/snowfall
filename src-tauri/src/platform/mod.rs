//! Per-OS window enumeration. `snapshot` returns visible top-level window rectangles
//! (screen pixels, top-most first) plus taskbar/dock, and whether a fullscreen app is active.

#[derive(Default, Clone)]
pub struct Snapshot {
    pub rects: Vec<[f64; 4]>, // x, y, w, h
    pub fullscreen: bool,
}

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::snapshot;

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::snapshot;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::snapshot;

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn snapshot(_own: u32) -> Snapshot {
    Snapshot::default()
}
