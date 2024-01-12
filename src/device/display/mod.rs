use screencapturekit::sc_display::SCDisplay;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

#[derive(Debug)]
pub struct Target {
    pub title: String,
    pub id: u32,
}

#[derive(Clone, Debug)]
pub enum Display {
    Mac(SCDisplay)
}

pub fn has_permission() -> bool {
    #[cfg(target_os = "macos")]
    return mac::has_permission();

    #[cfg(target_os = "windows")]
    return win::has_permission();
}

pub fn is_supported() -> bool {
    #[cfg(target_os = "macos")]
    return mac::is_supported();

    #[cfg(target_os = "windows")]
    return win::is_supported();
}

pub fn get_targets() -> Vec<Target> {
    #[cfg(target_os = "macos")]
    return mac::get_targets();

    #[cfg(target_os = "windows")]
    return win::get_targets();
}

pub fn get_main_display() -> Display {
    #[cfg(target_os = "macos")]
    {
        let sc_display = mac::get_main_display();
        return Display::Mac(sc_display);
    }
}

pub fn get_scale_factor(display_id: u32) -> u64 {
    #[cfg(target_os = "macos")]
    return mac::get_scale_factor(display_id);
}
