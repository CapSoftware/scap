#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

pub fn capture() {
    #[cfg(target_os = "macos")]
    mac::main();

    #[cfg(target_os = "windows")]
    windows::main();
}

pub fn is_supported() -> bool {
    // macOS: ScreenCaptureKit support
    // Windows:  Windows.Graphics.Capture
    true
}

pub fn has_permission() -> bool {
    // Check for screen recording permission
    // On macOS, check for accessibility permission too
    true
}

pub fn request_permission() -> bool {
    // Request screen recording permission
    // On macOS, request accessibility permission too
    true
}
