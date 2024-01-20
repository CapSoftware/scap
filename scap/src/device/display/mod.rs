
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

pub fn has_permission() -> bool {
    #[cfg(target_os = "macos")]
    return mac::has_permission();

    #[cfg(target_os = "windows")]
    return win::has_permission();

    #[cfg(target_os = "linux")]
    return linux::has_permission();
}

pub fn is_supported() -> bool {
    #[cfg(target_os = "macos")]
    return mac::is_supported();

    #[cfg(target_os = "windows")]
    return win::is_supported();

    #[cfg(target_os = "linux")]
    return linux::is_supported();
}

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
