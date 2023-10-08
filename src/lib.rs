#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

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
