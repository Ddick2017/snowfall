#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod platform;

use device_query::{DeviceQuery, DeviceState};
use serde::Serialize;
use std::{
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

#[derive(Serialize, Clone)]
struct Tick {
    cx: f64,
    cy: f64,
    fs: bool,
    paused: bool,
    rects: Vec<[f64; 4]>,
}

fn launcher() -> Result<auto_launch::AutoLaunch, String> {
    // inside an AppImage current_exe() points to a temporary mount, $APPIMAGE is the real file
    let path = std::env::var("APPIMAGE")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .ok_or("cannot find executable path")?;
    let path = path.to_string_lossy().to_string();
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name("Snowfall")
        .set_app_path(&path)
        .set_use_launch_agent(true)
        .build()
        .map_err(|e| e.to_string())
}

fn autostart(enable: bool) -> Result<(), String> {
    let al = launcher()?;
    let r = if enable { al.enable() } else { al.disable() };
    r.map_err(|e| e.to_string())
}

fn autostart_enabled() -> bool {
    launcher().ok().and_then(|a| a.is_enabled().ok()).unwrap_or(false)
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
    let _ = Command::new("pkill").args(["-xi", "snowfall"]).status();
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

    // X11 (or XWayland) is needed for a transparent click-through overlay and window enumeration
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("GDK_BACKEND", "x11");
        // avoids blank/crashing WebKit on NVIDIA and some other drivers
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    tauri::Builder::default()
        // a second launch does nothing (the running instance keeps going)
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
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
            // show() first: on Linux the GTK window must be realized before click-through can be set
            win.show()?;
            win.set_ignore_cursor_events(true)?; // clicks pass through to the apps below
            win.set_always_on_top(true)?;
            win.set_skip_taskbar(true)?;
            let _ = win.set_visible_on_all_workspaces(true);
            // Linux: WebKit has its own X sub-window, so also make every GTK widget click-through
            #[cfg(target_os = "linux")]
            if let Ok(gw) = win.gtk_window() {
                use gtk::prelude::*;
                fn click_through(w: &gtk::Widget) {
                    w.input_shape_combine_region(Some(&gtk::cairo::Region::create()));
                    if let Some(c) = w.downcast_ref::<gtk::Container>() {
                        for child in c.children() {
                            click_through(&child);
                        }
                    }
                }
                click_through(gw.upcast_ref::<gtk::Widget>());
            }

            // ---- tray / menu-bar icon: pause, launch at login, quit ----
            let paused = Arc::new(AtomicBool::new(false));
            let pause_item = CheckMenuItem::with_id(app, "pause", "Pause snow", true, false, None::<&str>)?;
            let auto_item =
                CheckMenuItem::with_id(app, "autostart", "Launch at login", true, autostart_enabled(), None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit Snowfall", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&pause_item, &auto_item, &sep, &quit_item])?;

            let (p_flag, p_item, a_item) = (paused.clone(), pause_item.clone(), auto_item.clone());
            let mut tray = TrayIconBuilder::new()
                .tooltip("Snowfall")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "pause" => p_flag.store(p_item.is_checked().unwrap_or(false), Ordering::Relaxed),
                    "autostart" => {
                        let want = a_item.is_checked().unwrap_or(false);
                        if autostart(want).is_err() {
                            let _ = a_item.set_checked(!want); // could not change it: revert the tick
                        }
                    }
                    _ => {}
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;

            // physical px -> CSS px. macOS reports points already.
            let (k, ox, oy) = if cfg!(target_os = "macos") { (1.0, 0.0, 0.0) } else { (1.0 / scale, mx, my) };

            let handle = app.handle().clone();
            let own = std::process::id();
            thread::spawn(move || {
                let ds = DeviceState::new();
                let debug = std::env::var_os("SNOWFALL_DEBUG").is_some();
                let (mut rects, mut fs, mut n) = (Vec::<[f64; 4]>::new(), false, 0u32);
                loop {
                    let is_paused = paused.load(Ordering::Relaxed);
                    if !is_paused && (fs || n % 6 == 0) {
                        let s = platform::snapshot(own);
                        rects = s
                            .rects
                            .iter()
                            .map(|r| [(r[0] - ox) * k, (r[1] - oy) * k, r[2] * k, r[3] * k])
                            .collect();
                        fs = s.fullscreen;
                    }
                    n = n.wrapping_add(1);
                    if debug && n % 30 == 0 {
                        eprintln!("fullscreen={fs} rects={rects:?}");
                    }
                    let (x, y) = ds.get_mouse().coords;
                    let _ = handle.emit(
                        "tick",
                        Tick {
                            cx: (x as f64 - ox) * k,
                            cy: (y as f64 - oy) * k,
                            fs,
                            paused: is_paused,
                            rects: rects.clone(),
                        },
                    );
                    let idle = fs || is_paused;
                    thread::sleep(Duration::from_millis(if idle { 500 } else { 33 }));
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running snowfall");
}
