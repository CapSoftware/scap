use std::cmp;
use std::sync::mpsc;

use screencapturekit::cm_sample_buffer::CMSampleBuffer;
use screencapturekit::sc_output_handler::SCStreamOutputType;
use screencapturekit::sc_stream_configuration::PixelFormat;
use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_display::SCDisplay,
    sc_error_handler::StreamErrorHandler,
    sc_output_handler::StreamOutput,
    sc_shareable_content::SCShareableContent,
    sc_stream::SCStream,
    sc_stream_configuration::SCStreamConfiguration,
};

use screencapturekit_sys::os_types::geometry::{CGPoint, CGRect, CGSize};
use screencapturekit_sys::sc_stream_frame_info::SCFrameStatus;

use crate::frame::{Frame, FrameType};
use crate::{capturer::Options, capturer::Resolution, targets};
use core_graphics_helmer_fork::display::CGDirectDisplayID;

mod pixelformat;

struct ErrorHandler;
impl StreamErrorHandler for ErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

pub struct Capturer {
    pub tx: mpsc::Sender<Frame>,
    pub output_type: FrameType,
}

impl Capturer {
    pub fn new(tx: mpsc::Sender<Frame>, output_type: FrameType) -> Self {
        Capturer { tx, output_type }
    }
}

impl StreamOutput for Capturer {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        match of_type {
            SCStreamOutputType::Screen => {
                let frame_status = &sample.frame_status;

                match frame_status {
                    SCFrameStatus::Complete => unsafe {
                        let frame;
                        match self.output_type {
                            FrameType::YUVFrame => {
                                let yuvframe = pixelformat::create_yuv_frame(sample).unwrap();
                                frame = Frame::YUVFrame(yuvframe);
                            }
                            FrameType::RGB => {
                                let rgbframe = pixelformat::create_rgb_frame(sample).unwrap();
                                frame = Frame::RGB(rgbframe);
                            }
                            FrameType::BGR0 => {
                                let bgrframe = pixelformat::create_bgr_frame(sample).unwrap();
                                frame = Frame::BGR0(bgrframe);
                            }
                            FrameType::BGRAFrame => {
                                let bgraframe = pixelformat::create_bgra_frame(sample).unwrap();
                                frame = Frame::BGRA(bgraframe);
                            }
                        }
                        self.tx.send(frame).unwrap_or(());
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

pub fn create_capturer(options: &Options, tx: mpsc::Sender<Frame>) -> SCStream {
    // TODO: identify targets to capture using options.targets
    // scap currently only captures the main display
    let display = targets::get_main_display();
    let sc_display = get_sc_display_from_id(display.id);

    let sc_shareable_content = SCShareableContent::current();

    let excluded_windows = sc_shareable_content
        .windows
        .into_iter()
        .filter(|window| {
            if let Some(excluded_window_names) = &options.excluded_windows {
                if let Some(current_window_name) = &window.title {
                    return excluded_window_names.contains(current_window_name);
                } else {
                    return false;
                }
            } else {
                return false;
            }
        })
        .collect();

    let params = InitParams::DisplayExcludingWindows(sc_display, excluded_windows);
    let filter = SCContentFilter::new(params);

    let source_rect = get_source_rect(options);
    let pixel_format = match options.output_type {
        FrameType::YUVFrame => PixelFormat::YCbCr420v,
        FrameType::BGR0 => PixelFormat::ARGB8888,
        FrameType::RGB => PixelFormat::ARGB8888,
        FrameType::BGRAFrame => PixelFormat::ARGB8888,
    };

    let [width, height] = get_output_frame_size(options);

    let stream_config = SCStreamConfiguration {
        width,
        height,
        source_rect,
        pixel_format,
        shows_cursor: options.show_cursor,
        ..Default::default()
    };

    let mut stream = SCStream::new(filter, stream_config, ErrorHandler);
    stream.add_output(
        Capturer::new(tx, options.output_type),
        SCStreamOutputType::Screen,
    );

    stream
}

pub fn get_output_frame_size(options: &Options) -> [u32; 2] {
    // TODO: this should be based on display from options.target, not main one
    let display = targets::get_main_display();
    let display_id = display.id;
    let scale = targets::get_scale_factor(display_id);

    let source_rect = get_source_rect(options);

    // Calculate the output height & width based on the required resolution
    // Output width and height need to be multiplied by scale (or dpi)
    let mut output_width = (source_rect.size.width as u32) * scale as u32;
    let mut output_height = (source_rect.size.height as u32) * scale as u32;
    // 1200x800
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

pub fn get_source_rect(options: &Options) -> CGRect {
    // TODO: this should be based on display from options.target, not main one
    let display = targets::get_main_display();
    let width = display.raw_handle.pixels_wide();
    let height = display.raw_handle.pixels_high();

    options
        .source_rect
        .as_ref()
        .map(|val| {
            let input_width = val.size.width + (val.size.width % 2.0);
            let input_height = val.size.height + (val.size.height % 2.0);

            CGRect {
                origin: CGPoint {
                    x: val.origin.x,
                    y: val.origin.y,
                },
                size: CGSize {
                    width: input_width as f64,
                    height: input_height as f64,
                },
            }
        })
        .unwrap_or_else(|| CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: width as f64,
                height: height as f64,
            },
        })
}

pub fn get_sc_display_from_id(id: CGDirectDisplayID) -> SCDisplay {
    SCShareableContent::current()
        .displays
        .into_iter()
        .find(|display| display.display_id == id)
        .unwrap_or_else(|| {
            SCShareableContent::current()
                .displays
                .get(0)
                .expect("couldn't find display")
                .to_owned()
        })
}
