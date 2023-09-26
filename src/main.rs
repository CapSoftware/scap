#[cfg(target_os = "windows")]
use windows::{
    core::*, Data::Xml::Dom::*, Win32::Foundation::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

extern crate ffmpeg_next;

fn main() -> Result<(), ()> {
    #[cfg(target_os = "macos")]
    {
        let devices = ffi::get_aperture_devices();
        println!("Devices: {}", devices);
    }

    let ffmpeg = ffmpeg_next::init().unwrap();

    println!("{:?}", ffmpeg);

    Ok(())
}

#[swift_bridge::bridge]
mod ffi {
    extern "Swift" {
        fn get_aperture_devices() -> String;
    }
}
