#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod windows;

fn main() {
    #[cfg(target_os = "macos")]
    {
        mac::main();
    }

    #[cfg(target_os = "windows")]
    {
        windows::main();
    }
}
