use core_graphics::{
    access::ScreenCaptureAccess,
    display::{CGDirectDisplayID, CGDisplay, CGMainDisplayID},
};
use itertools::Itertools;
use screencapturekit::{sc_display::SCDisplay, sc_shareable_content::SCShareableContent};
use sysinfo::System;

use super::{Display, DisplayPosition, DisplaySize, Target};

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
    let mut targets: Vec<Target> = Vec::new();

    let content = SCShareableContent::current();
    let displays = content.displays;

    for display in displays {
        // println!("Display: {:?}", display);
        let title = format!("Display {}", display.display_id); // TODO: get this from core-graphics

        let target = Target {
            id: display.display_id,
            title,
        };

        targets.push(target);
    }

    targets
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

pub fn get_all_displays() -> Vec<Display> {
    let content = SCShareableContent::current();
    let displays = content.displays;

    displays
        .iter()
        .map(|display| {
            let bounds = CGDisplay::new(display.display_id as CGDirectDisplayID).bounds();
            let scale = get_scale_factor(display.display_id);
            Display {
                id: display.display_id,
                physical_position: DisplayPosition {
                    x: bounds.origin.x,
                    y: bounds.origin.y,
                },
                size: DisplaySize {
                    width: bounds.size.width,
                    height: bounds.size.height,
                },
                scale_factor: scale,
            }
        })
        .collect_vec()
}

pub fn get_scale_factor(display_id: CGDirectDisplayID) -> u64 {
    let mode = CGDisplay::new(display_id).display_mode().unwrap();
    mode.pixel_width() / mode.width()
}
