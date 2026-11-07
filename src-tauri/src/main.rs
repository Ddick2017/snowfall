#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod platform;

use device_query::{DeviceQuery, DeviceState};
use serde::Serialize;
use std::{process::Command, thread, time::Duration};
use tauri::{Emitter, Manager};

#[derive(Serialize, Clone)]
struct Tick {
    cx: f64,
    cy: f64,
    fs: bool,
    rects: Vec<[f64; 4]>,
}

fn autostart(enable: bool) -> Result<(), String> {
    // inside an AppImage current_exe() points to a temporary mount, $APPIMAGE is the real file
    let path = std::env::var("APPIMAGE")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .ok_or("cannot find executable path")?;
    let path = path.to_string_lossy().to_string();
    let al = auto_launch::AutoLaunchBuilder::new()
        .set_app_name("Snowfall")
        .set_app_path(&path)
        .set_use_launch_agent(true)
        .build()
        .map_err(|e| e.to_string())?;
    let r = if enable { al.enable() } else { al.disable() };
    r.map_err(|e| e.to_string())
}

fn report(r: Result<(), String>) -> i32 {
    match r {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("snowfall: {e}");
            1
        }
    }
}

fn quit_running() {
    #[cfg(windows)]
    let _ = Command::new("taskkill").args(["/IM", "snowfall.exe", "/F"]).status();
    #[cfg(not(windows))]
    let _ = Command::new("pkill").args(["-if", "snowfall"]).status();
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--autostart-on") => std::process::exit(report(autostart(true))),
        Some("--autostart-off") => std::process::exit(report(autostart(false))),
        Some("--quit") => {
            quit_running();
            return;
        }
        _ => {}
    }

    // X11 (or XWayland) is needed for transparent click-through overlay and window enumeration
    #[cfg(target_os = "linux")]
    std::env::set_var("GDK_BACKEND", "x11");

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory); // no Dock icon

            let win = app.get_webview_window("main").ok_or("no main window")?;
            let mon = app.primary_monitor()?.ok_or("no monitor")?;
            let size = *mon.size();
            let (mx, my) = (mon.position().x as f64, mon.position().y as f64);
            let scale = mon.scale_factor();

            win.set_position(*mon.position())?;
            // 1px shorter than the screen so Windows does not treat the overlay as a "fullscreen app"
            win.set_size(tauri::PhysicalSize::new(size.width, size.height.saturating_sub(1)))?;
            win.set_ignore_cursor_events(true)?; // clicks pass through to the apps below
            win.set_always_on_top(true)?;
            win.set_skip_taskbar(true)?;
            let _ = win.set_visible_on_all_workspaces(true);
            win.show()?;

            // physical px -> CSS px. macOS reports points already.
            let (k, ox, oy) = if cfg!(target_os = "macos") { (1.0, 0.0, 0.0) } else { (1.0 / scale, mx, my) };

            let handle = app.handle().clone();
            let own = std::process::id();
            thread::spawn(move || {
                let ds = DeviceState::new();
                let (mut rects, mut fs, mut n) = (Vec::<[f64; 4]>::new(), false, 0u32);
                loop {
                    if fs || n % 6 == 0 {
                        let s = platform::snapshot(own);
                        rects = s
                            .rects
                            .iter()
                            .map(|r| [(r[0] - ox) * k, (r[1] - oy) * k, r[2] * k, r[3] * k])
                            .collect();
                        fs = s.fullscreen;
                    }
                    n = n.wrapping_add(1);
                    let (x, y) = ds.get_mouse().coords;
                    let _ = handle.emit(
                        "tick",
                        Tick { cx: (x as f64 - ox) * k, cy: (y as f64 - oy) * k, fs, rects: rects.clone() },
                    );
                    thread::sleep(Duration::from_millis(if fs { 500 } else { 33 }));
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running snowfall");
}
