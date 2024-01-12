use std::error::Error;
use std::sync::mpsc::Sender;
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFOEXW};
use windows_capture::{
    capture::{CaptureControl, WindowsCaptureHandler},
    frame::Frame,
    graphics_capture_api::{GraphicsCaptureApi, InternalCaptureControl},
    monitor::Monitor,
    settings::{ColorFormat, WindowsCaptureSettings},
    window::Window,
};

use crate::{Options, Target, TargetKind};

struct Capturer {
    tx: Sender<Vec<u8>>,
}

// IMPROVE: get user-friendly monitor name
fn get_monitor_name(h_monitor: HMONITOR) -> windows::core::Result<String> {
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    let success =
        unsafe { GetMonitorInfoW(h_monitor, &mut monitor_info as *mut _ as *mut _).as_bool() };

    if success {
        let len = monitor_info
            .szDevice
            .iter()
            .position(|&i| i == 0)
            .unwrap_or(0);
        let name = String::from_utf16(&monitor_info.szDevice[..len]).unwrap();

        let clean_name = match name.rfind('\\') {
            Some(index) => name.chars().skip(index + 1).collect(),
            None => name.to_string(),
        };

        Ok(clean_name)
    } else {
        Err(windows::core::Error::new(
            windows::core::HRESULT(0),
            "Failed to get monitor info".into(),
        ))
    }
}

impl WindowsCaptureHandler for Capturer {
    type Flags = String;

    fn new(message: Self::Flags) -> Result<Self, Box<dyn Error + Send + Sync>> {
        println!("Got The Message: {message}");

        // TODO: get tx from parent here
        Ok(Self { tx })
    }

    fn on_frame_arrived(
        &mut self,
        mut frame: Frame,
        _: InternalCaptureControl,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let buffer = frame.buffer();

        // FOR TESTING ONLY

        // Timestamp + Unique Value
        // self.frames += 1;
        // println!("frame: {}", self.frames);

        // Create an image and save frame to disk
        // let filename = format!("./test/frame-{}.png", self.frames);
        // println!("filename: {}", filename);
        // frame.save_as_image(&filename).unwrap();

        // Send frame buffer to parent
        self.tx.send(data).expect("Failed to send data");
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Closed");
        Ok(())
    }
}

fn remove_null_character(input: &str) -> String {
    match input.strip_suffix('\0') {
        Some(s) => s.to_string(),
        None => input.to_string(),
    }
}

pub fn create_recorder(options: &Options, tx: Sender<Vec<u8>>) -> CaptureControl {
    let settings = WindowsCaptureSettings::new(
        Monitor::primary().unwrap(),
        Some(true),
        Some(false),
        ColorFormat::Rgba8,
        "It Works".to_string(),
    )
    .unwrap();

    let stream = Recorder::start_free_threaded(settings);

    stream
}