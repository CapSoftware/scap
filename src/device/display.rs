use core_graphics::display::{CGMainDisplayID, CGDirectDisplayID, CGDisplay};
use screencapturekit::{
    sc_shareable_content::SCShareableContent,
    sc_display::SCDisplay
};


// TEMP: get main display
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