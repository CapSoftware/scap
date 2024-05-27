#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, Clone)]
pub struct Window {
    pub id: u32,
    pub title: String,

    #[cfg(target_os = "windows")]
    pub raw_handle: win::HWND,
}

#[derive(Debug, Clone)]
pub struct Display {
    pub id: u32,
    pub title: String,

    #[cfg(target_os = "windows")]
    pub raw_handle: win::HMONITOR,

    #[cfg(target_os = "macos")]
    pub raw_handle: mac::CGDisplay,
}

#[derive(Debug, Clone)]
pub enum Target {
    Window(Window),
    Display(Display),
}

/// Returns a list of targets that can be captured
pub fn get_targets() -> Vec<Target> {
    #[cfg(target_os = "macos")]
    return mac::get_targets();

    #[cfg(target_os = "windows")]
    return win::get_targets();

    #[cfg(target_os = "linux")]
    return linux::get_targets();
}

pub fn get_scale_factor(_display_id: u32) -> f64 {
    #[cfg(target_os = "macos")]
    return mac::get_scale_factor(_display_id);

    #[cfg(target_os = "windows")]
    return win::get_scale_factor();

    #[cfg(target_os = "linux")]
    return 1;
}

pub fn get_main_display() -> Display {
    #[cfg(target_os = "macos")]
    return mac::get_main_display();

    #[cfg(target_os = "windows")]
    return win::get_main_display();

    // #[cfg(target_os = "linux")]
    // return linux::get_main_display();
}
