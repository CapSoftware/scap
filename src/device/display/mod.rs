#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, Clone)]
pub enum Target {
    Display {
        id: u32,
        title: String,
        // origin of this display w.r.t. primary display
        physical_position: Point,
        size: Size,
        scale_factor: u64,
    },
    Window {},
}

/// Checks if process has permission to capture the screen
pub fn has_permission() -> bool {
    #[cfg(target_os = "macos")]
    return mac::has_permission();

    #[cfg(target_os = "windows")]
    return win::has_permission();

    #[cfg(target_os = "linux")]
    return linux::has_permission();
}

/// Prompts user to grant screen capturing permission to current process
pub fn request_permission() -> bool {
    #[cfg(target_os = "macos")]
    return mac::request_permission();

    // assume windows to be true
    #[cfg(target_os = "windows")]
    return true;

    // TODO: check if linux has permission system
    #[cfg(target_os = "linux")]
    return true;
}

/// Checks if scap is supported on the current system
pub fn is_supported() -> bool {
    #[cfg(target_os = "macos")]
    return mac::is_supported();

    #[cfg(target_os = "windows")]
    return win::is_supported();

    #[cfg(target_os = "linux")]
    return linux::is_supported();
}

/// Returns a list of screens that can be captured
pub fn get_targets() -> Vec<Target> {
    #[cfg(target_os = "macos")]
    return mac::get_targets();

    #[cfg(target_os = "windows")]
    return win::get_targets();

    #[cfg(target_os = "linux")]
    return linux::get_targets();
}

pub fn get_scale_factor(display_id: u32) -> u64 {
    #[cfg(target_os = "macos")]
    return mac::get_scale_factor(display_id);

    #[cfg(target_os = "windows")]
    return 1;

    #[cfg(target_os = "linux")]
    return 1; // TODO
}

#[cfg(target_os = "macos")]
use screencapturekit::sc_display::SCDisplay;

#[cfg(target_os = "macos")]
pub fn get_main_display() -> SCDisplay {
    #[cfg(target_os = "macos")]
    {
        let sc_display = mac::get_main_display();
        return sc_display;
    }
}

#[cfg(target_os = "macos")]
pub fn get_display(display_id: u32) -> SCDisplay {
    #[cfg(target_os = "macos")]
    {
        mac::get_display(display_id)
    }
}
#[cfg(target_os = "windows")]
use windows_capture::monitor::Monitor;

use crate::capturer::{Point, Size};

#[cfg(target_os = "windows")]
pub fn get_main_display() -> Monitor {
    #[cfg(target_os = "windows")]
    {
        let display = win::get_main_display();
        return display;
    }
}
