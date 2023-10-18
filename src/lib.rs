#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

pub enum TargetKind {
    Display,
    Window,
    Audio,
}

pub struct Target {
    kind: TargetKind,
    id: u32,
}

pub struct Options {
    fps: u32,
    targets: Vec<Target>,
    show_cursor: bool,
    show_highlight: bool,
}

pub fn capture() {
    #[cfg(target_os = "macos")]
    mac::main();

    #[cfg(target_os = "windows")]
    win::main();
}

pub fn is_supported() -> bool {
    #[cfg(target_os = "macos")]
    let access = mac::is_supported();

    #[cfg(target_os = "windows")]
    let access = win::is_supported();

    access
}

pub fn has_permission() -> bool {
    #[cfg(target_os = "macos")]
    let permission = mac::has_permission();

    #[cfg(target_os = "windows")]
    let permission = true; // TODO: check Windows permissions

    permission
}
