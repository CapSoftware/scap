use std::sync::mpsc;

use crate::{capturer::Options, frame::Frame};

pub struct WinStream {
    settings: WindowsCaptureSettings,
    tx: mpsc::Sender<Frame>,
}

impl WinStream {
    pub fn start_capture(&self) {
        Recorder::start_free_threaded(self.settings);
    }
}

pub fn create_capturer(options: &Options, tx: mpsc::Sender<Frame>) -> WinStream {
    let settings = WindowsCaptureSettings::new(
        Monitor::primary().unwrap(),
        Some(true),
        Some(false),
        ColorFormat::Rgba8,
        "It Works".to_string(),
    )
    .unwrap();

    return WinStream { settings, tx };
}