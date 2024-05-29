use crate::{
    capturer::{Area, Options, Point, Resolution, Size},
    frame::{BGRAFrame, Frame, FrameType},
    targets,
};
use std::cmp;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use windows_capture::{
    capture::{CaptureControl, GraphicsCaptureApiHandler},
    frame::Frame as WCFrame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor as WCMonitor,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
    window::Window as WCWindow,
};

#[derive(Debug)]
struct Capturer {
    pub tx: mpsc::Sender<Frame>,
    pub crop: Option<Area>,
}

pub struct WinStream {
    settings: Settings<FlagStruct, WCMonitor>,
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
        frame: &mut WCFrame,
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
    // TODO: get targets from options.targets

    let color_format = match options.output_type {
        FrameType::BGRAFrame => ColorFormat::Bgra8,
        _ => ColorFormat::Rgba8,
    };

    let show_cursor = match options.show_cursor {
        true => CursorCaptureSettings::WithCursor,
        false => CursorCaptureSettings::WithoutCursor,
    };

    let show_highlight = match options.show_highlight {
        true => DrawBorderSettings::WithBorder,
        false => DrawBorderSettings::WithoutBorder,
    };

    let settings = Settings::new(
        WCMonitor::primary().unwrap(),
        show_cursor,
        show_highlight,
        color_format,
        FlagStruct {
            tx,
            crop: Some(get_crop_area(options)),
        },
    );

    return WinStream {
        settings,
        capture_control: None,
    };
}

pub fn get_output_frame_size(options: &Options) -> [u32; 2] {
    // TODO: this should be based on display from options.target, not main one
    let display = targets::get_main_display();
    let display_id = display.id;
    let scale_factor = targets::get_scale_factor(display_id);

    let source_rect = get_crop_area(options);

    let mut output_width = (source_rect.size.width as u32) * scale_factor as u32;
    let mut output_height = (source_rect.size.height as u32) * scale_factor as u32;

    match options.output_resolution {
        Resolution::Captured => {}
        _ => {
            let [resolved_width, resolved_height] = options
                .output_resolution
                .value((source_rect.size.width as f32) / (source_rect.size.height as f32));
            // 1280 x 853
            output_width = cmp::min(output_width, resolved_width);
            output_height = cmp::min(output_height, resolved_height);
        }
    }

    output_width -= output_width % 2;
    output_height -= output_height % 2;

    [output_width, output_height]
}

pub fn get_crop_area(options: &Options) -> Area {
    let display = targets::get_main_display();
    let display_raw = WCMonitor::from_raw_hmonitor(display.raw_handle.0);

    let width_result = display_raw.width();
    let height_result = display_raw.height();

    let width = width_result.unwrap_or(0);
    let height = height_result.unwrap_or(0);

    options
        .crop_area
        .as_ref()
        .map(|val| {
            let input_width = val.size.width + val.size.width % 2.0;
            let input_height = val.size.height + val.size.height % 2.0;
            Area {
                origin: val.origin.clone(),
                size: Size {
                    width: input_width as f64,
                    height: input_height as f64,
                },
            }
        })
        .unwrap_or_else(|| Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: width as f64,
                height: height as f64,
            },
        })
}
