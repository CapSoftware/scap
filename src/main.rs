#[cfg(target_os = "windows")]
use windows::Graphics::Capture;

fn main() -> Result<(), ()> {
    #[cfg(target_os = "macos")]
    {
        let devices = ffi::get_aperture_devices();
        println!("Devices: {}", devices);
    }

    #[cfg(target_os = "windows")]
    {
        let supported = Capture::GraphicsCaptureSession::IsSupported().unwrap();

        println!("Supported: {}", supported);
    }

    Ok(())
}

#[swift_bridge::bridge]
#[cfg(target_os = "macos")]
mod ffi {
    // extern "Rust" {
    //     fn rust_double_number(num: i64) -> i64;
    // }

    extern "Swift" {
        fn get_aperture_devices() -> String;
    }
}

// fn rust_double_number(num: i64) -> i64 {
//     println!("Rust double function called...");

//     num * 2
// }
