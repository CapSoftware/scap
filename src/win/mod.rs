use std::time::Instant;
use windows::Graphics::Capture;
use windows_capture::{
    capture::{WindowsCaptureHandler, WindowsCaptureSettings},
    frame::Frame,
    monitor::Monitor,
    window::Window,
};

use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFOEXW};

use crate::{Target, TargetKind};

struct Recorder {
    frames: usize,
    last_output: Instant,
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
        Self {
            frames: 0,
            last_output: Instant::now(),
        }
    }

    fn on_frame_arrived(&mut self, frame: &Frame) {
        self.frames += 1;

        println!("frame: {}", self.frames);
        println!("size: {}x{}", frame.width(), frame.height());

        // TODO: encode the frames received here into a video
    }

    fn on_closed(&mut self) {
        println!("Closed");
    }
}

pub fn main() {
    let settings = WindowsCaptureSettings::new(Monitor::get_primary(), true, false, ());

    println!("Capture started. Press Enter to stop.");
    Recorder::start(settings).unwrap();

    // TODO: figure out threading mechanism here
}

pub fn is_supported() -> bool {
    Capture::GraphicsCaptureSession::IsSupported().unwrap()
}

pub fn has_permission() -> bool {
    // TODO: add correct permission mechanism here
    true
}

pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    let displays = Monitor::list_monitors().unwrap();
    let windows = Window::get_windows().unwrap();

    for display in displays {
        let id = display;

        // TODO: get name;

        let name = get_monitor_name(display).unwrap();
        println!("name: {}", name);

        let target = Target {
            kind: TargetKind::Display,
            id: 2,
            name: name,
        };
        targets.push(target);
    }

    // TODO: complete windows implementation

    // for window in windows {
    //     let id = window;
    //     let target = Target {
    //         kind: TargetKind::Window,
    //         id,
    //     };
    //     targets.push(target);
    // }

    targets
}
