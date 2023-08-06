#[cfg(target_os = "windows")]
use windows::{
    core::*, Data::Xml::Dom::*, Win32::Foundation::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

fn main() -> Result<(), ()> {
    #[cfg(target_os = "macos")]
    {
        let devices = ffi::get_aperture_devices();
        println!("Devices: {}", devices);
    }

    println!("Hello, world!");

    #[cfg(target_os = "windows")]
    {
        let doc = XmlDocument::new()?;
        doc.LoadXml(h!("<html>hello world</html>"))?;

        let root = doc.DocumentElement()?;
        assert!(root.NodeName()? == "html");
        assert!(root.InnerText()? == "hello world");

        unsafe {
            let event = CreateEventW(None, true, false, None)?;
            SetEvent(event).ok()?;
            WaitForSingleObject(event, 0);
            CloseHandle(event).ok()?;

            MessageBoxA(None, s!("Ansi"), s!("Caption"), MB_OK);
            MessageBoxW(None, w!("Wide"), w!("Caption"), MB_OK);
        }
    }

    Ok(())
}

#[swift_bridge::bridge]
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
