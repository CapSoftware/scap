use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::{cmp, sync::Arc};

use cidre::mach;
use cidre::sc::StreamDelegateImpl;
use cidre::{
    arc, cg, cm, cv, define_obj_type, dispatch, ns, objc,
    sc::{self, StreamDelegate, StreamOutput, StreamOutputImpl},
};
use futures::executor::block_on;

use crate::frame::{AudioFormat, AudioFrame, Frame, FrameType, VideoFrame};
use crate::targets::Target;
use crate::{
    capturer::{Area, Options, Point, Resolution, Size},
    frame::BGRAFrame,
    targets,
};

use super::ChannelItem;

pub(crate) mod ext;
mod pixel_buffer;
mod pixelformat;

struct ErrorHandlerInner {
    error_flag: Arc<AtomicBool>,
}

define_obj_type!(
    pub ErrorHandler + StreamDelegateImpl,
    ErrorHandlerInner,
    ERROR_HANDLER
);

impl sc::stream::Delegate for ErrorHandler {}

#[objc::add_methods]
impl sc::stream::DelegateImpl for ErrorHandler {
    extern "C" fn impl_stream_did_stop_with_err(
        &mut self,
        _cmd: Option<&objc::Sel>,
        stream: &sc::Stream,
        error: &ns::Error,
    ) {
        eprintln!("Screen capture error occurred.");
        self.inner_mut()
            .error_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

#[repr(C)]
pub struct CapturerInner {
    pub tx: mpsc::Sender<ChannelItem>,
}

define_obj_type!(pub Capturer + StreamOutputImpl, CapturerInner, CAPTURER);

impl sc::stream::Output for Capturer {}

#[objc::add_methods]
impl sc::stream::OutputImpl for Capturer {
    extern "C" fn impl_stream_did_output_sample_buf(
        &mut self,
        _cmd: Option<&objc::Sel>,
        _stream: &sc::Stream,
        sample_buf: &mut cm::SampleBuf,
        kind: sc::OutputType,
    ) {
        let _ = self.inner_mut().tx.send((sample_buf.retained(), kind));
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum CreateCapturerError {
    #[error("{0}")]
    OtherNative(#[from] arc::R<ns::Error>),
    #[error("Window with title '{0}' not found")]
    WindowNotFound(String),
    #[error("Display with title '{0}' not found")]
    DisplayNotFound(String),
}

pub(crate) fn create_capturer(
    options: &Options,
    tx: mpsc::Sender<ChannelItem>,
    error_flag: Arc<AtomicBool>,
) -> Result<(arc::R<Capturer>, arc::R<ErrorHandler>, arc::R<sc::Stream>), CreateCapturerError> {
    // If no target is specified, capture the main display
    let target = options
        .target
        .clone()
        .unwrap_or_else(|| Target::Display(targets::get_main_display()));

    let shareable_content = block_on(sc::ShareableContent::current())?;

    let filter = match target {
        Target::Window(window) => {
            let windows = shareable_content.windows();

            // Get SCWindow from window id
            let sc_window = windows
                .iter()
                .find(|sc_win| sc_win.id() == window.id)
                .ok_or_else(|| CreateCapturerError::WindowNotFound(window.title))?;

            // Return a DesktopIndependentWindow
            // https://developer.apple.com/documentation/screencapturekit/sccontentfilter/3919804-init
            sc::ContentFilter::with_desktop_independent_window(sc_window)
        }
        Target::Display(display) => {
            let displays = shareable_content.displays();
            // Get SCDisplay from display id
            let sc_display = displays
                .iter()
                .find(|sc_dis| sc_dis.display_id() == display.raw_handle)
                .ok_or_else(|| CreateCapturerError::DisplayNotFound(display.title))?;

            match &options.excluded_targets {
                None => sc::ContentFilter::with_display_excluding_windows(
                    &sc_display,
                    &ns::Array::new(),
                ),
                Some(excluded_targets) => {
                    let windows = shareable_content.windows();
                    let excluded_windows = windows
                        .iter()
                        .filter(|window| {
                            excluded_targets
                                .iter()
                                .any(|excluded_target| match excluded_target {
                                    Target::Window(excluded_window) => {
                                        excluded_window.id == window.id()
                                    }
                                    _ => false,
                                })
                        })
                        .collect::<Vec<_>>();

                    sc::ContentFilter::with_display_excluding_windows(
                        &sc_display,
                        &ns::Array::from_slice(&excluded_windows),
                    )
                }
            }
        }
    };

    let crop_area = get_crop_area(options);

    let source_rect = cg::Rect {
        origin: cg::Point {
            x: crop_area.origin.x,
            y: crop_area.origin.y,
        },
        size: cg::Size {
            width: crop_area.size.width,
            height: crop_area.size.height,
        },
    };

    let pixel_format = match options.output_type {
        FrameType::YUVFrame => cv::PixelFormat::_420V,
        FrameType::BGR0 => cv::PixelFormat::_32_BGRA,
        FrameType::RGB => cv::PixelFormat::_32_BGRA,
        FrameType::BGRAFrame => cv::PixelFormat::_32_BGRA,
    };

    let [width, height] = get_output_frame_size(options);

    let mut stream_config = sc::StreamCfg::new();
    stream_config.set_width(width as usize);
    stream_config.set_height(height as usize);
    stream_config.set_src_rect(source_rect);
    stream_config.set_pixel_format(pixel_format);
    stream_config.set_shows_cursor(options.show_cursor);
    stream_config.set_minimum_frame_interval(cm::Time {
        value: 1,
        scale: options.fps as i32,
        epoch: 0,
        flags: cm::TimeFlags::VALID,
    });
    stream_config.set_captures_audio(options.captures_audio);

    let error_handler = ErrorHandler::with(ErrorHandlerInner { error_flag });
    let stream = sc::Stream::with_delegate(&filter, &stream_config, error_handler.as_ref());

    let capturer = CapturerInner { tx };

    let queue = dispatch::Queue::serial_with_ar_pool();

    let capturer = Capturer::with(capturer);

    if options.captures_audio {
        stream
            .add_stream_output(capturer.as_ref(), sc::OutputType::Audio, Some(&queue))
            .unwrap();
    }

    stream
        .add_stream_output(capturer.as_ref(), sc::OutputType::Screen, Some(&queue))
        .unwrap();

    Ok((capturer, error_handler, stream))
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
    mut sample: arc::R<cm::SampleBuf>,
    of_type: sc::stream::OutputType,
    output_type: FrameType,
) -> Option<Frame> {
    let system_time = std::time::SystemTime::now();
    let system_mach_time = mach::abs_time();

    let frame_cm_time = sample.pts();
    let frame_mach_time = cm::Clock::convert_host_time_to_sys_units(frame_cm_time);

    let mach_time_diff = if frame_mach_time > system_mach_time {
        (frame_mach_time - system_mach_time) as i64
    } else {
        -((system_mach_time - frame_mach_time) as i64)
    };

    // Convert mach time difference to nanoseconds
    let mach_timebase = mach::TimeBaseInfo::new();
    let nanos_diff = (mach_time_diff * mach_timebase.numer as i64) / mach_timebase.denom as i64;

    // Calculate frame SystemTime
    let frame_system_time = if nanos_diff >= 0 {
        system_time + std::time::Duration::from_nanos(nanos_diff as u64)
    } else {
        system_time - std::time::Duration::from_nanos((-nanos_diff) as u64)
    };

    match of_type {
        sc::stream::OutputType::Screen => {
            let attaches = sample.attaches(false).and_then(|a| {
                let mut iter = a.iter();
                iter.next()
            })?;

            match attaches
                .get(sc::FrameInfo::status().as_cf())?
                .as_number()
                .to_i32()
                .unwrap()
            {
                0 => unsafe {
                    return Some(Frame::Video(match output_type {
                        FrameType::YUVFrame => {
                            let yuvframe =
                                pixelformat::create_yuv_frame(sample.as_mut(), frame_system_time)
                                    .unwrap();
                            VideoFrame::YUVFrame(yuvframe)
                        }
                        FrameType::RGB => {
                            let rgbframe =
                                pixelformat::create_rgb_frame(sample.as_mut(), frame_system_time)
                                    .unwrap();
                            VideoFrame::RGB(rgbframe)
                        }
                        FrameType::BGR0 => {
                            let bgrframe =
                                pixelformat::create_bgr_frame(sample.as_mut(), frame_system_time)
                                    .unwrap();
                            VideoFrame::BGR0(bgrframe)
                        }
                        FrameType::BGRAFrame => {
                            let bgraframe =
                                pixelformat::create_bgra_frame(sample.as_mut(), frame_system_time)
                                    .unwrap();
                            VideoFrame::BGRA(bgraframe)
                        }
                    }));
                },
                1 => {
                    // Quick hack - just send an empty frame, and the caller can figure out how to handle it
                    if let FrameType::BGRAFrame = output_type {
                        return Some(Frame::Video(VideoFrame::BGRA(BGRAFrame {
                            display_time: frame_system_time,
                            width: 0,
                            height: 0,
                            data: vec![],
                        })));
                    }
                }
                _ => {}
            };

            None
        }
        sc::stream::OutputType::Audio => {
            let list = sample.audio_buf_list::<2>().ok()?;
            let mut bytes = Vec::<u8>::new();

            for buffer in list.list().buffers {
                bytes.extend(unsafe {
                    std::slice::from_raw_parts(buffer.data, buffer.data_bytes_size as usize)
                });
            }

            return Some(Frame::Audio(AudioFrame::new(
                AudioFormat::F32,
                2,
                false,
                bytes,
                sample.num_samples() as usize,
                48_000,
                frame_system_time,
            )));
        }
        _ => None,
    }
}
