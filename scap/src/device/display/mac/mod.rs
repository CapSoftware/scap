use core_graphics_helmer_fork::{
    access::ScreenCaptureAccess,
    display::{CGDirectDisplayID, CGDisplay, CGMainDisplayID},
};
use screencapturekit::{sc_display::SCDisplay, sc_shareable_content::SCShareableContent};
use sysinfo::System;

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
    let content = SCShareableContent::current();
    let displays = content.displays;

    let main_display_id = unsafe { CGMainDisplayID() };
    let main_display = displays
        .iter()
        .find(|display| display.display_id == main_display_id)
        .unwrap_or_else(|| {
            panic!("Main display not found");
        });

    main_display.to_owned()
}

pub fn get_scale_factor(display_id: CGDirectDisplayID) -> u64 {
    let mode = CGDisplay::new(display_id).display_mode().unwrap();
    mode.pixel_width() / mode.width()
}
