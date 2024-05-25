#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, Clone)]
pub struct Target {
    pub title: String,
    pub id: u32,
}

#[derive(Debug, Clone)]
pub struct DisplayPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct DisplaySize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct Display {
    pub id: u32,
    // origin of this display w.r.t. primary display
    pub physical_position: DisplayPosition,
    pub size: DisplaySize,
    pub scale_factor: u64,
}

/// Returns all displays present in the system
pub fn get_all_displays() -> Vec<Display> {
    #[cfg(target_os = "macos")]
    return mac::get_all_displays();

    #[cfg(target_os = "windows")]
    return vec![]; // TODO; Unimplemneted

    #[cfg(target_os = "linux")]
    return vec![]; // TODO; Unimplemneted
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

#[cfg(target_os = "windows")]
pub fn get_main_display() -> Monitor {
    #[cfg(target_os = "windows")]
    {
        let display = win::get_main_display();
        return display;
    }
}
