#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod windows;

pub fn capture() {
    #[cfg(target_os = "macos")]
    mac::main().expect("Error!");

    #[cfg(target_os = "windows")]
    windows::main();
}
