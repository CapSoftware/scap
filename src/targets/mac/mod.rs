use core_graphics_helmer_fork::display::{CGDirectDisplayID, CGMainDisplayID};
use screencapturekit::{sc_display::SCDisplay, sc_shareable_content::SCShareableContent};

use super::Target;

pub use core_graphics_helmer_fork::display::CGDisplay;

pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    let content = SCShareableContent::current();

    // Add displays to targets
    for display in content.displays {
        // println!("Display: {:?}", display);

        // TODO: get this from core-graphics
        let title = format!("Display {}", display.display_id);

        let target = Target::Display(super::Display {
            title,
            id: display.display_id,
            raw_handle: CGDisplay::new(display.display_id),
        });

        targets.push(target);
    }

    // Add windows to targets
    for window in content.windows {
        if window.title.is_none() {
            continue;
        }

        let title = window.title.expect("Window title not found");

        let target = Target::Window(super::Window {
            title,
            id: window.window_id,
        });

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
