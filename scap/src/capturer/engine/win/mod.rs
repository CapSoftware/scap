use std::error::Error;
use std::sync::mpsc;

use crate::{
    capturer::{Options,CGRect},
    frame::{BGRFrame, Frame, RGBxFrame},
};
use windows::{Wdk::System::SystemServices::OkControl, Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFOEXW}};
use windows_capture::{
    capture::{CaptureControl, WindowsCaptureHandler},
    frame::Frame as Wframe,
    graphics_capture_api::{GraphicsCaptureApi, InternalCaptureControl},
    monitor::Monitor,
    settings::{ColorFormat, Settings},
    window::Window,
};

#[derive(Debug)]
struct Capturer {
    pub tx: mpsc::Sender<Frame>,
    pub crop: Option<CGRect>,
}

impl Capturer {
    pub fn new(tx: mpsc::Sender<Frame>) -> Self {
        println!("I am here inside impl_capturer_new");
        Capturer { tx, crop: None }
    }

    pub fn with_crop(mut self, crop: Option<CGRect>) -> Self {
        self.crop = crop;
        self
    }
}

pub struct WinStream {
    settings: Settings<FlagStruct>,
    capture_control: Option<CaptureControl<Capturer, Box<dyn std::error::Error + Send + Sync>>>,
}

impl WindowsCaptureHandler for Capturer {
    type Flags = FlagStruct;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(flagValues: Self::Flags) -> Result<Self, Self::Error> {
        println!("I am here inside WindowsCaptureHandler new");
        Ok(Self { tx:flagValues.tx, crop:flagValues.crop })
    }

    fn on_frame_arrived(
        &mut self,
        mut frame: &mut Wframe,
        _: InternalCaptureControl,
    ) -> Result<(), Self::Error> {

        match &self.crop {
            Some(cropped_area) => {

                // get the cropped area
                let start_x = cropped_area.origin.x as u32;
                let start_y = cropped_area.origin.y as u32;
                let end_x = (cropped_area.origin.x + cropped_area.size.width) as u32;
                let end_y = (cropped_area.origin.y + cropped_area.size.height) as u32;

                // crop the frame
                let mut cropped_buffer = frame.buffer_crop(start_x, start_y, end_x, end_y)
                    .expect("Failed to crop buffer");

                println!("Frame Arrived: {}x{} and padding = {}",
                    cropped_buffer.width(),
                    cropped_buffer.height(),
                    cropped_buffer.has_padding(),
                );

                // get raw frame buffer
                let raw_frame_buffer = match cropped_buffer.as_raw_nopadding_buffer() {
                    Ok(buffer) => buffer,
                    Err(_) => return Err(("Failed to get raw buffer").into()),
    
                };

                let bgr_frame = BGRFrame {
                    display_time: 0,
                    width: cropped_area.size.width as i32,
                    height: cropped_area.size.height as i32,
                    data: raw_frame_buffer.to_vec(),
                };

                self.tx.send(Frame::BGR0(bgr_frame))
                    .expect("Failed to send data");
            }
            None => {
                println!("Frame Arrived: {}x{}",
                    frame.width(),
                    frame.height(),
                );

                // get raw frame buffer
                let mut frame_buffer = frame.buffer().unwrap();
                let raw_frame_buffer = frame_buffer.as_raw_buffer();
                let frame_data = raw_frame_buffer.to_vec();
                
                let bgr_frame = BGRFrame {
                    display_time: 0,
                    width: frame.width() as i32,
                    height: frame.height() as i32,
                    data: frame_data,
                };

                self.tx.send(Frame::BGR0(bgr_frame))
                    .expect("Failed to send data");
            }
        }
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Closed");
        Ok(())
    }
}

impl WinStream {
    pub fn start_capture(&mut self) {

        let capture_control = Capturer::start_free_threaded(self.settings.clone()).unwrap();
        self.capture_control = Some(capture_control);
    }

    pub fn stop_capture(&mut self) {
        let capture_control = self.capture_control.take().unwrap();
        let _ = capture_control.stop();
    }
}

#[derive(Clone, Debug)]
struct FlagStruct {
    pub tx: mpsc::Sender<Frame>,
    pub crop: Option<CGRect>,
}

pub fn create_capturer(
    options: &Options,
    tx: mpsc::Sender<Frame>,
) -> WinStream {
    let settings = Settings::new(
        Monitor::primary().unwrap(),
        Some(true),
        None,
        ColorFormat::Rgba8,
        FlagStruct { tx, crop: options.source_rect.clone() },
    
    ).unwrap();

    return WinStream {
        settings,
        capture_control: None,
    };
}