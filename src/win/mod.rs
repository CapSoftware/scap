use std::time::Instant;
use windows::Graphics::Capture;
use windows_capture::{
    capture::{WindowsCaptureHandler, WindowsCaptureSettings},
    frame::Frame,
    monitor::Monitor,
    window::Window,
};

use crate::{Target, TargetKind};

struct Recorder {
    frames: usize,
    last_output: Instant,
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

pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    let displays = Monitor::list_monitors().unwrap();
    let windows = Window::get_windows().unwrap();

    for display in displays {
        let id = display;
        // TODO: get name;

        let target = Target {
            kind: TargetKind::Display,
            id,
        };
        targets.push(target);
    }

    for window in windows {
        let id = window;
        let target = Target {
            kind: TargetKind::Window,
            id,
        };
        targets.push(target);
    }

    targets
}
