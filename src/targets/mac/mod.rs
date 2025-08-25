use cidre::{cg, ns, sc};
use cocoa::appkit::{NSApp, NSScreen};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSRect, NSString, NSUInteger};
use futures::executor::block_on;
use objc::{msg_send, sel, sel_impl};

use crate::engine::mac::ext::DirectDisplayIdExt;

use super::{Display, Target};

fn get_display_name(display_id: cg::DirectDisplayId) -> String {
    unsafe {
        // Get all screens
        let screens: id = NSScreen::screens(nil);
        let count: u64 = msg_send![screens, count];

        for i in 0..count {
            let screen: id = msg_send![screens, objectAtIndex: i];
            let device_description: id = msg_send![screen, deviceDescription];
            let display_id_number: id = msg_send![device_description, objectForKey: NSString::alloc(nil).init_str("NSScreenNumber")];
            let display_id_number: u32 = msg_send![display_id_number, unsignedIntValue];

            if display_id_number == display_id.0 {
                let localized_name: id = msg_send![screen, localizedName];
                let name: *const i8 = msg_send![localized_name, UTF8String];
                return std::ffi::CStr::from_ptr(name)
                    .to_string_lossy()
                    .into_owned();
            }
        }

        format!("Unknown Display {}", display_id.0)
    }
}

pub fn get_all_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    let content = block_on(sc::ShareableContent::current()).unwrap();

    // Add displays to targets
    for display in content.displays().iter() {
        let id = display.display_id();

        let title = get_display_name(id);

        let target = Target::Display(super::Display {
            id: id.0,
            title,
            raw_handle: id,
        });

        targets.push(target);
    }

    // Add windows to targets
    for window in content.windows().iter() {
        let id = window.id();
        let title = window
            .title()
            // on intel chips we can have Some but also a null pointer for some reason
            .filter(|v| !unsafe { v.utf8_chars_ar().is_null() });

        let target = Target::Window(super::Window {
            id,
            title: title.map(|v| v.to_string()).unwrap_or_default(),
            raw_handle: id,
        });
        targets.push(target);
    }

    targets
}

pub fn get_main_display() -> Display {
    let id = cg::direct_display::Id::main();
    let title = get_display_name(id);

    Display {
        id: id.0,
        title,
        raw_handle: id,
    }
}

pub fn get_scale_factor(target: &Target) -> f64 {
    match target {
        Target::Window(window) => {
            // Get the window's frame to determine which display it's on
            let content = match block_on(sc::ShareableContent::current()) {
                Ok(c) => c,
                Err(_) => return 1.0, // fallback on SC failures to avoid panic
            };

            // Find the window in ScreenCaptureKit
            if let Some(sc_window) = content
                .windows()
                .iter()
                .find(|w| w.id() == window.raw_handle)
            {
                let window_frame = sc_window.frame();
                let window_center_x = window_frame.origin.x + window_frame.size.width / 2.0;
                let window_center_y = window_frame.origin.y + window_frame.size.height / 2.0;

                // Find which display contains the center of the window
                for display in content.displays().iter() {
                    let display_id = display.display_id();
                    let bounds = display_id.bounds();

                    if window_center_x >= bounds.origin.x
                        && window_center_x < bounds.origin.x + bounds.size.width
                        && window_center_y >= bounds.origin.y
                        && window_center_y < bounds.origin.y + bounds.size.height
                    {
                        // Found the display containing the window's center
                        if let Some(mode) = display_id.display_mode() {
                            return (mode.pixel_width() as f64) / mode.width() as f64;
                        }
                    }
                }
            }

            // Fallback: if we can't determine the display or get scale factor, use 1.0
            1.0
        }
        Target::Display(display) => {
            let mode = display.raw_handle.display_mode().unwrap();
            (mode.pixel_width() / mode.width()) as f64
        }
    }
}

pub fn get_target_dimensions(target: &Target) -> (u64, u64) {
    match target {
        Target::Window(window) => {
            let cg_win_id = window.raw_handle;

            // Use ScreenCaptureKit directly to get window dimensions
            let content = match block_on(sc::ShareableContent::current()) {
                Ok(c) => c,
                Err(_) => return (800, 600), // conservative fallback on SC failures
            };
            if let Some(sc_window) = content.windows().iter().find(|w| w.id() == cg_win_id) {
                let frame = sc_window.frame(); // points
                let scale = get_scale_factor(target); // uses SC as well; safe to reuse
                let w_px = (frame.size.width * scale).round() as u64;
                let h_px = (frame.size.height * scale).round() as u64;
                return (w_px, h_px);
            }
            // Fallback to default dimensions if window not found
            (800, 600)
        }
        Target::Display(display) => {
            let mode = display.raw_handle.display_mode().unwrap();
            (mode.width(), mode.height())
        }
    }
}
