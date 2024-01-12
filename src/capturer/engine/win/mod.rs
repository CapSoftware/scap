use std::sync::mpsc;
use std::error::Error;

use crate::{capturer::Options, frame::Frame};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFOEXW};
use windows_capture::{
    capture::{CaptureControl, WindowsCaptureHandler},
    frame::Frame as Wframe,
    graphics_capture_api::{GraphicsCaptureApi, InternalCaptureControl},
    monitor::Monitor,
    settings::{ColorFormat, WindowsCaptureSettings},
    window::Window,
};

struct Capturer {
    pub tx: mpsc::Sender<Frame>,
}

impl Capturer {
    pub fn new(tx: mpsc::Sender<Frame>) -> Self {
        Capturer { tx }
    }
}

pub struct WinStream {
    settings: WindowsCaptureSettings<mpsc::Sender<Frame>>,
}

impl WindowsCaptureHandler for Capturer {
    type Flags = mpsc::Sender<Frame>;

    fn new(tx: Self::Flags) -> Result<Self, Box<dyn Error + Send + Sync>> {

        Ok(Self { tx })
    }

    fn on_frame_arrived(
        &mut self,
        mut frame: Wframe,
        _: InternalCaptureControl,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {

        let frame_buffer= frame.buffer().unwrap().as_raw_buffer();
        let frame_data = frame_buffer.to_vec();
        self.tx.send(Frame::BGR0(frame_data)).expect("Failed to send data");
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Closed");
        Ok(())
    }
}


impl WinStream {
    pub fn start_capture(&self) {
        // TODO: Prevent cloning the transmitter
        Capturer::start_free_threaded(self.settings.clone());
    }
}

pub fn create_capturer(options: &Options, tx: mpsc::Sender<Frame>) -> WinStream {
    let settings = WindowsCaptureSettings::new(
        Monitor::primary().unwrap(),
        Some(true),
        Some(false),
        ColorFormat::Rgba8,
        tx,
    )
    .unwrap();

    return WinStream { settings };
}