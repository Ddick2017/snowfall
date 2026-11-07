use super::Snapshot;
use std::cell::RefCell;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

struct X {
    root: Window,
    sw: f64,
    sh: f64,
    list: u32,
    state: u32,
    fullscreen: u32,
    hidden: u32,
    pid: u32,
    desk: u32,
    curdesk: u32,
    frame: u32,
    workarea: u32,
    wtype: u32,
    normal: u32,
    dialog: u32,
    skip_taskbar: u32,
    c: RustConnection,
}

thread_local! { static CONN: RefCell<Option<X>> = RefCell::new(None); }

fn atom(c: &RustConnection, name: &str) -> Option<u32> {
    Some(c.intern_atom(false, name.as_bytes()).ok()?.reply().ok()?.atom)
}

fn init() -> Option<X> {
    let (c, n) = RustConnection::connect(None).ok()?;
    let (root, sw, sh) = {
        let s = &c.setup().roots[n];
        (s.root, s.width_in_pixels as f64, s.height_in_pixels as f64)
    };
    Some(X {
        root,
        sw,
        sh,
        list: atom(&c, "_NET_CLIENT_LIST_STACKING")?,
        state: atom(&c, "_NET_WM_STATE")?,
        fullscreen: atom(&c, "_NET_WM_STATE_FULLSCREEN")?,
        hidden: atom(&c, "_NET_WM_STATE_HIDDEN")?,
        pid: atom(&c, "_NET_WM_PID")?,
        desk: atom(&c, "_NET_WM_DESKTOP")?,
        curdesk: atom(&c, "_NET_CURRENT_DESKTOP")?,
        frame: atom(&c, "_NET_FRAME_EXTENTS")?,
        workarea: atom(&c, "_NET_WORKAREA")?,
        wtype: atom(&c, "_NET_WM_WINDOW_TYPE")?,
        normal: atom(&c, "_NET_WM_WINDOW_TYPE_NORMAL")?,
        dialog: atom(&c, "_NET_WM_WINDOW_TYPE_DIALOG")?,
        skip_taskbar: atom(&c, "_NET_WM_STATE_SKIP_TASKBAR")?,
        c,
    })
}

fn prop(x: &X, w: Window, p: u32, ty: AtomEnum, len: u32) -> Option<Vec<u32>> {
    let r = x.c.get_property(false, w, p, ty, 0, len).ok()?.reply().ok()?;
    let v: Vec<u32> = r.value32()?.collect();
    Some(v)
}

fn first(v: Option<Vec<u32>>) -> Option<u32> {
    v.and_then(|v| v.first().copied())
}

fn query(x: &X, own: u32) -> Option<Snapshot> {
    let cur = first(prop(x, x.root, x.curdesk, AtomEnum::CARDINAL, 1));
    let list = prop(x, x.root, x.list, AtomEnum::WINDOW, 4096)?; // bottom -> top
    let mut snap = Snapshot::default();

    for &w in list.iter().rev() {
        if first(prop(x, w, x.pid, AtomEnum::CARDINAL, 1)) == Some(own) {
            continue;
        }
        let st = prop(x, w, x.state, AtomEnum::ATOM, 32).unwrap_or_default();
        if st.contains(&x.hidden) {
            continue;
        }
        if let (Some(d), Some(c)) = (first(prop(x, w, x.desk, AtomEnum::CARDINAL, 1)), cur) {
            if d != 0xFFFF_FFFF && d != c {
                continue; // other workspace
            }
        }
        if st.contains(&x.fullscreen) {
            snap.fullscreen = true;
            continue;
        }
        // only real application windows: skips the desktop-icons layer, docks, panels, our own helpers
        let ty = prop(x, w, x.wtype, AtomEnum::ATOM, 8).unwrap_or_default();
        if !ty.is_empty() && !ty.iter().any(|a| *a == x.normal || *a == x.dialog) {
            continue;
        }
        if st.contains(&x.skip_taskbar) {
            continue;
        }
        let Some(g) = x.c.get_geometry(w).ok().and_then(|c| c.reply().ok()) else { continue };
        let Some(t) = x.c.translate_coordinates(w, x.root, 0, 0).ok().and_then(|c| c.reply().ok()) else { continue };
        let mut fe = prop(x, w, x.frame, AtomEnum::CARDINAL, 4).unwrap_or_default();
        if fe.len() < 4 {
            fe = vec![0; 4];
        }
        let (l, r, top, b) = (fe[0] as f64, fe[1] as f64, fe[2] as f64, fe[3] as f64);
        let rect = [
            t.dst_x as f64 - l,
            t.dst_y as f64 - top,
            g.width as f64 + l + r,
            g.height as f64 + top + b,
        ];
        if rect[2] >= 80.0 && rect[3] >= 40.0 {
            snap.rects.push(rect);
        }
    }

    // panel/taskbar at the bottom: the line where the work area ends
    if let Some(wa) = prop(x, x.root, x.workarea, AtomEnum::CARDINAL, 4) {
        if wa.len() == 4 {
            let bottom = (wa[1] + wa[3]) as f64;
            if bottom < x.sh - 2.0 {
                snap.rects.push([0.0, bottom, x.sw, x.sh - bottom]);
            }
        }
    }
    Some(snap)
}

pub fn snapshot(own: u32) -> Snapshot {
    CONN.with(|cell| {
        let mut b = cell.borrow_mut();
        if b.is_none() {
            *b = init();
        }
        let r = b.as_ref().and_then(|x| query(x, own));
        if r.is_none() {
            *b = None; // reconnect next time
        }
        r.unwrap_or_default()
    })
}
