use core_graphics_helmer_fork::{
    access::ScreenCaptureAccess,
    display::{CGDirectDisplayID, CGDisplay, CGMainDisplayID},
};
use itertools::Itertools;
use screencapturekit::{sc_display::SCDisplay, sc_shareable_content::SCShareableContent};
use sysinfo::System;

use crate::capturer::{Point, Size};

use super::Target;

pub fn has_permission() -> bool {
    ScreenCaptureAccess::default().preflight()
}

pub fn request_permission() -> bool {
    ScreenCaptureAccess::default().request()
}

pub fn is_supported() -> bool {
    let os_version = System::os_version()
        .expect("Failed to get macOS version")
        .as_bytes()
        .to_vec();

    let min_version: Vec<u8> = "12.3\n".as_bytes().to_vec();

    os_version >= min_version
}

pub fn get_targets() -> Vec<Target> {
    return get_displays()
        .into_iter()
        .chain(get_windows().into_iter())
        .collect();
}

pub fn get_main_display() -> SCDisplay {
    let main_display_id = unsafe { CGMainDisplayID() };
    get_display(main_display_id)
}

pub fn get_display(display_id: CGDirectDisplayID) -> SCDisplay {
    let content = SCShareableContent::current();
    let displays = content.displays;

    let display = displays
        .iter()
        .find(|display| display.display_id == display_id)
        .unwrap_or_else(|| {
            panic!("Main display not found");
        });
    display.to_owned()
}

pub fn get_scale_factor(display_id: CGDirectDisplayID) -> u64 {
    let mode = CGDisplay::new(display_id).display_mode().unwrap();
    mode.pixel_width() / mode.width()
}

/// returns all physical screens presently connected.
fn get_displays() -> Vec<Target> {
    SCShareableContent::current()
        .displays
        .iter()
        .map(|display| {
            let bounds = CGDisplay::new(display.display_id as CGDirectDisplayID).bounds();
            let scale = get_scale_factor(display.display_id);
            let scale_f64 = scale as f64;
            Target::Display {
                id: display.display_id,
                title: format!("Display {}", display.display_id), // TODO: get this from core-graphics
                physical_position: Point {
                    x: bounds.origin.x * scale_f64,
                    y: bounds.origin.y * scale_f64,
                },
                size: Size {
                    width: bounds.size.width * scale_f64,
                    height: bounds.size.height * scale_f64,
                },
                scale_factor: scale,
            }
        })
        .collect_vec()
}

/// return all open windows across all displays.
fn get_windows() -> Vec<Target> {
    return Vec::new();
}
