#[cfg(target_os = "windows")]
mod win;

#[tokio::main]
async fn main() -> Result<(), ()> {
    #[cfg(target_os = "macos")]
    {
        let devices = ffi::get_aperture_devices();
        println!("Devices: {}", devices);
    }

    #[cfg(target_os = "windows")]
    win::main().await;

    Ok(())
}

#[swift_bridge::bridge]
#[cfg(target_os = "macos")]
mod ffi {
    extern "Swift" {
        fn get_aperture_devices() -> String;
    }
}
