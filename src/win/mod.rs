use std::sync::mpsc::Sender;
use windows_capture::{
    capture::{CaptureControl, WindowsCaptureHandler},
    frame::Frame,
    graphics_capture_api::{GraphicsCaptureApi, InternalCaptureControl},
    monitor::Monitor,
    settings::{ColorFormat, Settings},
    window::Window,
};

use crate::{utils, Options, Target, TargetKind};

pub struct Capturer {
    tx: Sender<Vec<u8>>, // Not a good idea to copy the entire buffer
}

impl WindowsCaptureHandler for Capturer {
    type Flags = Sender<Vec<u8>>;
    type Error = anyhow::Error;

    fn new(tx: Self::Flags) -> Result<Self, Self::Error> {
        // TODO: get tx from parent here
        Ok(Self { tx })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
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
        let mut data = frame.buffer()?;
        let data = data.as_raw_buffer();
        // let data = data.as_raw_nopadding_buffer()?; // Or without padding?

        self.tx.send(data.to_vec()).expect("Failed to send data"); // still not a good idea to copy the entire buffer
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
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

pub fn create_recorder(
    options: &Options,
    tx: Sender<Vec<u8>>,
) -> utils::Result<CaptureControl<Capturer, anyhow::Error>> {
    let settings = Settings::new(
        Monitor::primary().unwrap(),
        Some(true),
        Some(false),
        ColorFormat::Rgba8,
        tx,
    )?;

    let capture_control = Capturer::start_free_threaded(settings)?;

    Ok(capture_control)
}

pub fn is_supported() -> bool {
    GraphicsCaptureApi::is_supported().expect("Failed to check support")
}

pub fn has_permission() -> bool {
    // TODO: add correct permission mechanism here
    // Its a win32 app, so it should be fine
    true
}

pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    let displays = Monitor::enumerate().expect("Failed to enumerate monitors");

    for display in displays {
        let id = display;
        let title = display.device_string().unwrap_or(String::from("Unknown"));

        let target = Target {
            id: 2,
            title,
            kind: TargetKind::Display,
        };
        targets.push(target);
    }

    let windows = Window::enumerate().expect("Failed to enumerate windows");
    for window in windows {
        let handle = window.as_raw_hwnd();

        let title = window
            .title()
            .unwrap()
            .strip_suffix('\0')
            .unwrap()
            .to_string();

        let target = Target {
            id: 3,
            kind: TargetKind::Window,
            title,
        };
        targets.push(target);
    }

    targets
}
