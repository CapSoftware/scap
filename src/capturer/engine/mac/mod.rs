use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::{cmp, sync::Arc};

use core_foundation::error::CFError;
use core_graphics::display::{CGPoint, CGRect, CGSize};
use core_media_rs::cm_time::CMTime;
use pixelformat::get_pts_in_nanoseconds;
use screencapturekit::{
    output::{
        sc_stream_frame_info::{SCFrameStatus, SCStreamFrameInfo},
        CMSampleBuffer,
    },
    shareable_content::SCShareableContent,
    stream::{
        configuration::{pixel_format::PixelFormat, SCStreamConfiguration},
        content_filter::SCContentFilter,
        delegate_trait::SCStreamDelegateTrait,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
        SCStream,
    },
};

use crate::frame::{AudioFormat, AudioFrame, Frame, FrameType, VideoFrame};
use crate::targets::Target;
use crate::{
    capturer::{Area, Options, Point, Resolution, Size},
    frame::BGRAFrame,
    targets,
};

use super::ChannelItem;

mod apple_sys;
mod pixel_buffer;
mod pixelformat;

struct ErrorHandler {
    error_flag: Arc<AtomicBool>,
}

impl SCStreamDelegateTrait for ErrorHandler {
    fn did_stop_with_error(&self, _stream: SCStream, _error: CFError) {
        eprintln!("Screen capture error occurred.");
        self.error_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Clone)]
pub struct Capturer {
    pub tx: mpsc::Sender<ChannelItem>,
}

impl Capturer {
    pub fn new(tx: mpsc::Sender<ChannelItem>) -> Self {
        Capturer { tx }
    }
}

impl SCStreamOutputTrait for Capturer {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        self.tx.send((sample, of_type)).unwrap_or(());
    }
}

pub fn create_capturer(
    options: &Options,
    tx: mpsc::Sender<ChannelItem>,
    error_flag: Arc<AtomicBool>,
) -> Result<SCStream, CFError> {
    // If no target is specified, capture the main display
    let target = options
        .target
        .clone()
        .unwrap_or_else(|| Target::Display(targets::get_main_display()));

    let sc_shareable_content = SCShareableContent::get().unwrap();

    let filter = match target {
        Target::Window(window) => {
            // Get SCWindow from window id
            let sc_window = sc_shareable_content
                .windows()
                .into_iter()
                .find(|sc_win| sc_win.window_id() == window.id)
                .unwrap();

            // Return a DesktopIndependentWindow
            // https://developer.apple.com/documentation/screencapturekit/sccontentfilter/3919804-init
            SCContentFilter::new().with_desktop_independent_window(&sc_window)
        }
        Target::Display(display) => {
            // Get SCDisplay from display id
            let sc_display = sc_shareable_content
                .displays()
                .into_iter()
                .find(|sc_dis| sc_dis.display_id() == display.id)
                .unwrap();

            match &options.excluded_targets {
                None => SCContentFilter::new().with_display_excluding_windows(&sc_display, &[]),
                Some(excluded_targets) => {
                    let windows = sc_shareable_content.windows();
                    let excluded_windows = windows
                        .iter()
                        .filter(|window| {
                            excluded_targets
                                .iter()
                                .any(|excluded_target| match excluded_target {
                                    Target::Window(excluded_window) => {
                                        excluded_window.id == window.window_id()
                                    }
                                    _ => false,
                                })
                        })
                        .collect::<Vec<_>>();

                    SCContentFilter::new()
                        .with_display_excluding_windows(&sc_display, excluded_windows.as_slice())
                }
            }
        }
    };

    let crop_area = get_crop_area(options);

    let source_rect = CGRect {
        origin: CGPoint {
            x: crop_area.origin.x,
            y: crop_area.origin.y,
        },
        size: CGSize {
            width: crop_area.size.width,
            height: crop_area.size.height,
        },
    };

    let pixel_format = match options.output_type {
        FrameType::YUVFrame => PixelFormat::YCbCr_420v,
        FrameType::BGR0 => PixelFormat::BGRA,
        FrameType::RGB => PixelFormat::BGRA,
        FrameType::BGRAFrame => PixelFormat::BGRA,
    };

    let [width, height] = get_output_frame_size(options);

    let stream_config = SCStreamConfiguration::new()
        .set_width(width)?
        .set_height(height)?
        .set_source_rect(source_rect)?
        .set_pixel_format(pixel_format)?
        .set_shows_cursor(options.show_cursor)?
        .set_minimum_frame_interval(&CMTime {
            value: 1,
            timescale: options.fps as i32,
            epoch: 0,
            flags: 1,
        })?
        .set_captures_audio(options.captures_audio)?;

    let mut stream =
        SCStream::new_with_delegate(&filter, &stream_config, ErrorHandler { error_flag });

    let capturer = Capturer::new(tx);

    if options.captures_audio {
        stream.add_output_handler(capturer.clone(), SCStreamOutputType::Audio);
    }

    stream.add_output_handler(capturer, SCStreamOutputType::Screen);

    Ok(stream)
}

pub fn get_output_frame_size(options: &Options) -> [u32; 2] {
    let target = options
        .target
        .clone()
        .unwrap_or_else(|| Target::Display(targets::get_main_display()));

    let scale_factor = targets::get_scale_factor(&target);
    let source_rect = get_crop_area(options);

    // Calculate the output height & width based on the required resolution
    // Output width and height need to be multiplied by scale (or dpi)
    let mut output_width = (source_rect.size.width as u32) * (scale_factor as u32);
    let mut output_height = (source_rect.size.height as u32) * (scale_factor as u32);
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

pub fn get_crop_area(options: &Options) -> Area {
    let target = options
        .target
        .clone()
        .unwrap_or_else(|| Target::Display(targets::get_main_display()));

    let (width, height) = targets::get_target_dimensions(&target);

    options
        .crop_area
        .as_ref()
        .map(|val| {
            let input_width = val.size.width + (val.size.width % 2.0);
            let input_height = val.size.height + (val.size.height % 2.0);

            Area {
                origin: Point {
                    x: val.origin.x,
                    y: val.origin.y,
                },
                size: Size {
                    width: input_width,
                    height: input_height,
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

pub fn process_sample_buffer(
    sample: CMSampleBuffer,
    of_type: SCStreamOutputType,
    output_type: FrameType,
) -> Option<Frame> {
    match of_type {
        SCStreamOutputType::Screen => {
            let info = SCStreamFrameInfo::from_sample_buffer(&sample).unwrap();
            let frame_status = info.status();

            match frame_status {
                SCFrameStatus::Complete | SCFrameStatus::Started => unsafe {
                    return Some(Frame::Video(match output_type {
                        FrameType::YUVFrame => {
                            let yuvframe = pixelformat::create_yuv_frame(sample).unwrap();
                            VideoFrame::YUVFrame(yuvframe)
                        }
                        FrameType::RGB => {
                            let rgbframe = pixelformat::create_rgb_frame(sample).unwrap();
                            VideoFrame::RGB(rgbframe)
                        }
                        FrameType::BGR0 => {
                            let bgrframe = pixelformat::create_bgr_frame(sample).unwrap();
                            VideoFrame::BGR0(bgrframe)
                        }
                        FrameType::BGRAFrame => {
                            let bgraframe = pixelformat::create_bgra_frame(sample).unwrap();
                            VideoFrame::BGRA(bgraframe)
                        }
                    }));
                },
                SCFrameStatus::Idle => {
                    // Quick hack - just send an empty frame, and the caller can figure out how to handle it
                    if let FrameType::BGRAFrame = output_type {
                        return Some(Frame::Video(VideoFrame::BGRA(BGRAFrame {
                            display_time: get_pts_in_nanoseconds(&sample),
                            width: 0,
                            height: 0,
                            data: vec![],
                        })));
                    }
                }
                _ => {}
            }
        }
        SCStreamOutputType::Audio => {
            let list = sample.get_audio_buffer_list().unwrap();
            let mut bytes = Vec::<u8>::new();

            for buffer in list.buffers() {
                bytes.extend(buffer.data());
            }

            return Some(Frame::Audio(AudioFrame::new(
                AudioFormat::F32,
                2,
                false,
                bytes,
                sample.get_num_samples() as usize,
                48_000,
            )));
        }
    };

    None
}
