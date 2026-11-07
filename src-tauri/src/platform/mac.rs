use super::Snapshot;
use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::display::CGDisplay;
use core_graphics::window::{
    copy_window_info, kCGNullWindowID, kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
};

type Dict = CFDictionary<CFString, CFType>;

fn num(d: &Dict, key: &str) -> Option<f64> {
    let k = CFString::new(key);
    d.find(&k)?.downcast::<CFNumber>()?.to_f64()
}

fn text(d: &Dict, key: &str) -> Option<String> {
    let k = CFString::new(key);
    Some(d.find(&k)?.downcast::<CFString>()?.to_string())
}

pub fn snapshot(own: u32) -> Snapshot {
    let mut snap = Snapshot::default();
    let screen = CGDisplay::main().bounds();
    let (sw, sh) = (screen.size.width, screen.size.height);

    let Some(arr) = copy_window_info(kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements, kCGNullWindowID)
    else {
        return snap;
    };

    for item in arr.iter() {
        let d: Dict = unsafe { CFDictionary::wrap_under_get_rule(*item as CFDictionaryRef) };
        if num(&d, "kCGWindowOwnerPID").map(|p| p as u32) == Some(own) {
            continue;
        }
        let layer = num(&d, "kCGWindowLayer").unwrap_or(-1.0) as i32;
        let Some(bv) = d.find(&CFString::new("kCGWindowBounds")) else { continue };
        let b: Dict = unsafe { CFDictionary::wrap_under_get_rule(bv.as_CFTypeRef() as CFDictionaryRef) };
        let (Some(x), Some(y), Some(w), Some(h)) = (num(&b, "X"), num(&b, "Y"), num(&b, "Width"), num(&b, "Height"))
        else {
            continue;
        };

        if layer == 0 {
            if x <= 0.0 && y <= 0.0 && w >= sw && h >= sh {
                snap.fullscreen = true; // an app owns the whole screen incl. menu bar area
            } else if w >= 80.0 && h >= 40.0 {
                snap.rects.push([x, y, w, h]);
            }
        } else if layer == 20 && text(&d, "kCGWindowOwnerName").as_deref() == Some("Dock") && h < 300.0 && w < sw {
            snap.rects.push([x, y, w, h]); // the Dock
        }
    }
    snap
}
