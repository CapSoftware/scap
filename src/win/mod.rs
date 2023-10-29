use std::{path::PathBuf, time::Instant};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFOEXW};
use windows_capture::{
    capture::WindowsCaptureHandler, frame::Frame, graphics_capture_api::GraphicsCaptureApi,
    monitor::Monitor, settings::WindowsCaptureSettings, window::Window,
};

use crate::{Target, TargetKind};

struct Recorder {
    frames: usize,
}

fn get_monitor_name(h_monitor: HMONITOR) -> windows::core::Result<String> {
    let mut monitor_info = MONITORINFOEXW::default();

    monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    let success =
        unsafe { GetMonitorInfoW(h_monitor, &mut monitor_info as *mut _ as *mut _).as_bool() };

    if success {
        let name = unsafe {
            let len = monitor_info
                .szDevice
                .iter()
                .position(|&i| i == 0)
                .unwrap_or(0);
            String::from_utf16(&monitor_info.szDevice[..len])?
        };
        Ok(name)
    } else {
        Err(windows::core::Error::new(
            windows::core::HRESULT(0),
            "Failed to get monitor info".into(),
        ))
    }
}

impl WindowsCaptureHandler for Recorder {
    type Flags = ();

    fn new(_: Self::Flags) -> Self {
        Self { frames: 0 }
    }

    fn on_frame_arrived(&mut self, frame: Frame) {
        self.frames += 1;

        println!("frame: {}", self.frames);
        println!("size: {}x{}", frame.width(), frame.height());

        // println!("buffer: {:?}", frame.buffer());

        let filename = format!("./test/test-frame-{}.png", self.frames);
        println!("filename: {}", filename);

        frame.save_as_image(&filename).unwrap();

        // TODO: encode the frames received here into a video
    }

    fn on_closed(&mut self) {
        println!("Closed");
    }
}

pub fn main() {
    let settings =
        WindowsCaptureSettings::new(Monitor::primary(), Some(true), Some(false), ()).unwrap();

    println!("Capture started. Press Enter to stop.");
    Recorder::start(settings).unwrap();

    // TODO: figure out threading mechanism here
}

pub fn is_supported() -> bool {
    GraphicsCaptureApi::is_supported().expect("Failed to check support")
}

pub fn has_permission() -> bool {
    // TODO: add correct permission mechanism here
    true
}

pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    let displays = Monitor::enumerate().expect("Failed to enumerate monitors");

    for display in displays {
        let id = display;

        // TODO: get name;

        let name = get_monitor_name(display).unwrap();
        println!("name: {}", name);

        let target = Target {
            kind: TargetKind::Display,
            id: 2,
            name,
        };
        targets.push(target);
    }

    // TODO: complete windows implementation
    let windows = Window::enumerate().expect("Failed to enumerate windows");
    for window in windows {
        let id = window;
        let target = Target {
            kind: TargetKind::Window,
            name: "".into(),
            id: 3,
        };
        targets.push(target);
    }

    targets
}
