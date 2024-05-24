use crate::{
    capturer::{Area, Options, Point, Resolution, Size},
    device::display::{self},
    frame::{BGRAFrame, Frame, FrameType},
};
use std::cmp;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use windows::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, LOGPIXELSX, LOGPIXELSY};
use windows_capture::{
    capture::{CaptureControl, GraphicsCaptureApiHandler},
    frame::Frame as Wframe,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
};

#[derive(Debug)]
struct Capturer {
    pub tx: mpsc::Sender<Frame>,
    pub crop: Option<Area>,
}

impl Capturer {
    pub fn new(tx: mpsc::Sender<Frame>) -> Self {
        Capturer { tx, crop: None }
    }

    pub fn with_crop(mut self, crop: Option<Area>) -> Self {
        self.crop = crop;
        self
    }
}

pub struct WinStream {
    settings: Settings<FlagStruct, Monitor>,
    capture_control: Option<CaptureControl<Capturer, Box<dyn std::error::Error + Send + Sync>>>,
}

impl GraphicsCaptureApiHandler for Capturer {
    type Flags = FlagStruct;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(flag_values: Self::Flags) -> Result<Self, Self::Error> {
        Ok(Self {
            tx: flag_values.tx,
            crop: flag_values.crop,
        })
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
                let mut cropped_buffer = frame
                    .buffer_crop(start_x, start_y, end_x, end_y)
                    .expect("Failed to crop buffer");

                // get raw frame buffer
                let raw_frame_buffer = match cropped_buffer.as_raw_nopadding_buffer() {
                    Ok(buffer) => buffer,
                    Err(_) => return Err(("Failed to get raw buffer").into()),
                };

                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Failed to get current time")
                    .as_nanos() as u64;

                let bgr_frame = BGRAFrame {
                    display_time: current_time,
                    width: cropped_area.size.width as i32,
                    height: cropped_area.size.height as i32,
                    data: raw_frame_buffer.to_vec(),
                };

                self.tx
                    .send(Frame::BGRA(bgr_frame))
                    .expect("Failed to send data");
            }
            None => {
                // get raw frame buffer
                let mut frame_buffer = frame.buffer().unwrap();
                let raw_frame_buffer = frame_buffer.as_raw_buffer();
                let frame_data = raw_frame_buffer.to_vec();
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Failed to get current time")
                    .as_nanos() as u64;
                let bgr_frame = BGRAFrame {
                    display_time: current_time,
                    width: frame.width() as i32,
                    height: frame.height() as i32,
                    data: frame_data,
                };

                self.tx
                    .send(Frame::BGRA(bgr_frame))
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
    pub crop: Option<Area>,
}

pub fn create_capturer(options: &Options, tx: mpsc::Sender<Frame>) -> WinStream {
    let color_format = match options.output_type {
        FrameType::BGRAFrame => ColorFormat::Bgra8,
        _ => ColorFormat::Rgba8,
    };

    let settings = Settings::new(
        Monitor::primary().unwrap(),
        CursorCaptureSettings::Default,
        DrawBorderSettings::Default,
        color_format,
        FlagStruct {
            tx,
            crop: Some(get_source_rect(options)),
        },
    );

    return WinStream {
        settings,
        capture_control: None,
    };
}

pub fn get_output_frame_size(options: &Options) -> [u32; 2] {
    let scale = get_scale_factor();
    let source_rect = get_source_rect(options);

    let mut output_width = source_rect.size.width as u32;
    let mut output_height = source_rect.size.height as u32;

    match options.output_resolution {
        Resolution::Captured => {}
        _ => {
            let [resolved_width, resolved_height] = options
                .output_resolution
                .value((source_rect.size.width as f32) / (source_rect.size.height as f32));
            output_width = cmp::min(output_width, resolved_width);
            output_height = cmp::min(output_height, resolved_height);
        }
    }

    if output_width % 2 == 1 {
        output_width -= 1;
    }

    if output_height % 2 == 1 {
        output_height -= 1;
    }
    [output_width, output_height]
}

pub fn get_source_rect(options: &Options) -> Area {
    let display = display::get_main_display();
    let width_result = display.width();
    let height_result = display.height();

    let width: u32 = match width_result {
        Ok(val) => val,
        Err(_) => 0,
    };
    let height = match height_result {
        Ok(val) => val,
        Err(_) => 0,
    };

    let source_rect = match &options.source_rect {
        Some(val) => {
            let input_width = if (val.size.width as i64) % 2 == 0 {
                val.size.width as i64
            } else {
                (val.size.width as i64) + 1
            };
            let input_height = if (val.size.height as i64) % 2 == 0 {
                val.size.height as i64
            } else {
                (val.size.height as i64) + 1
            };
            Area {
                origin: Point {
                    x: val.origin.x,
                    y: val.origin.y,
                },
                size: Size {
                    width: input_width as f64,
                    height: input_height as f64,
                },
            }
        }
        None => Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: width as f64,
                height: height as f64,
            },
        },
    };

    source_rect
}

fn get_scale_factor() -> f64 {
    unsafe {
        let hdc = GetDC(None);

        let dpi_x = GetDeviceCaps(hdc, LOGPIXELSX);
        let dpi_y = GetDeviceCaps(hdc, LOGPIXELSY);

        ReleaseDC(None, hdc);

        let scale_x = dpi_x as f64 / 96.0;
        let scale_y = dpi_y as f64 / 96.0;
        let scale = (scale_x + scale_y) / 2.0;

        return scale;
    }
}
