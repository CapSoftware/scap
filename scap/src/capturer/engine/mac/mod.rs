use std::sync::mpsc;
use std::{cmp, mem, slice};

use screencapturekit::cm_sample_buffer::CMSampleBuffer;
use screencapturekit::sc_output_handler::SCStreamOutputType;
use screencapturekit::sc_stream_configuration::PixelFormat;
use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_error_handler::StreamErrorHandler,
    sc_output_handler::StreamOutput,
    sc_shareable_content::SCShareableContent,
    sc_stream::SCStream,
    sc_stream_configuration::SCStreamConfiguration,
};
use screencapturekit_sys::cm_sample_buffer_ref::{
    CMSampleBufferGetImageBuffer, CMSampleBufferGetSampleAttachmentsArray,
};
use screencapturekit_sys::os_types::geometry::{CGPoint, CGRect, CGSize};
use screencapturekit_sys::sc_stream_frame_info::SCFrameStatus;

use crate::frame::{
    convert_bgra_to_rgb, get_cropped_data, remove_alpha_channel, BGRAFrame, BGRFrame, Frame,
    FrameType, RGBFrame, YUVFrame,
};
use crate::{
    capturer::Options,
    capturer::Resolution,
    device::display::{self},
};
use apple_sys::{
    CoreMedia::{
        CFDictionaryGetValue, CFDictionaryRef, CFNumberGetValue, CFNumberType_kCFNumberSInt64Type,
        CFTypeRef,
    },
    ScreenCaptureKit::{SCFrameStatus_SCFrameStatusComplete, SCStreamFrameInfoStatus},
};
use core_graphics::display::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_video_sys::{
    CVPixelBufferGetBaseAddress, CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRow,
    CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferGetHeight, CVPixelBufferGetHeightOfPlane,
    CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth, CVPixelBufferGetWidthOfPlane,
    CVPixelBufferLockBaseAddress, CVPixelBufferRef, CVPixelBufferUnlockBaseAddress,
};

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
                                let yuvframe = create_yuv_frame(sample).unwrap();
                                frame = Frame::YUVFrame(yuvframe);
                            }
                            FrameType::RGB => {
                                let rgbframe = create_rgb_frame(sample).unwrap();
                                frame = Frame::RGB(rgbframe);
                            }
                            FrameType::BGR0 => {
                                let bgrframe = create_bgr_frame(sample).unwrap();
                                frame = Frame::BGR0(bgrframe);
                            }
                            FrameType::BGRAFrame => {
                                let bgraframe = create_bgra_frame(sample).unwrap();
                                frame = Frame::BGRA(bgraframe);
                            }
                            _ => {
                                panic!("Unimplemented Output format");
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
    let display = display::get_main_display();
    let display_id = display.display_id;

    let scale = display::get_scale_factor(display_id) as u32;

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

    let params = InitParams::DisplayExcludingWindows(display, excluded_windows);
    let filter = SCContentFilter::new(params);

    let source_rect = get_source_rect(options);
    let pixel_format = match options.output_type {
        FrameType::YUVFrame => PixelFormat::YCbCr420v,
        FrameType::BGR0 => PixelFormat::ARGB8888,
        FrameType::RGB => PixelFormat::ARGB8888,
        FrameType::BGRAFrame => PixelFormat::ARGB8888,
    };

    let [output_width, output_height] = get_output_frame_size(options);

    let stream_config = SCStreamConfiguration {
        shows_cursor: options.show_cursor,
        width: output_width as u32,
        height: output_height as u32,
        source_rect,
        pixel_format,
        ..Default::default()
    };

    let mut stream = SCStream::new(filter, stream_config, ErrorHandler);
    stream.add_output(
        Capturer::new(tx, options.output_type),
        SCStreamOutputType::Screen,
    );

    stream
}

pub fn ycbcr_to_rgb(
    y_data: &[u8],
    cbcr_data: &[u8],
    width: usize,
    height: usize,
    stride: usize,
) -> Vec<u8> {
    let mut rgb_data = Vec::with_capacity(width * height * 3);
    let row = width + stride;

    for h in 0..height {
        for w in 0..width {
            let y_idx = h * row + w;
            let uv_idx = (h / 2) * row + w - w % 2;

            // let y = y_data[y_idx] as f32;
            // let cb = cbcr_data[uv_idx] as f32 - 128.0;
            // let cr = cbcr_data[uv_idx + 1] as f32 - 128.0;

            // NOTE: The following values adjust for contrast and range
            let y = (y_data[y_idx] as f32 - 16.0) * (255.0 / (235.0 - 16.0));
            let cb = (cbcr_data[uv_idx] as f32 - 16.0) * (255.0 / (240.0 - 16.0)) - 128.0;
            let cr = (cbcr_data[uv_idx + 1] as f32 - 16.0) * (255.0 / (240.0 - 16.0)) - 128.0;

            let r = (y + 1.402 * cr).max(0.0).min(255.0) as u8;
            let g = (y - 0.344136 * cb - 0.714136 * cr).max(0.0).min(255.0) as u8;
            let b = (y + 1.772 * cb).max(0.0).min(255.0) as u8;

            rgb_data.push(r);
            rgb_data.push(g);
            rgb_data.push(b);
        }
    }
    rgb_data
}

pub unsafe fn create_yuv_frame(sample_buffer: CMSampleBuffer) -> Option<YUVFrame> {
    // Check that the frame status is complete
    let buffer_ref = &(*sample_buffer.sys_ref);
    {
        let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer_ref, 0);
        if attachments.is_null() || CFArrayGetCount(attachments as CFArrayRef) == 0 {
            return None;
        }
        let attachment = CFArrayGetValueAtIndex(attachments as CFArrayRef, 0) as CFDictionaryRef;
        let frame_status_ref = CFDictionaryGetValue(
            attachment as CFDictionaryRef,
            SCStreamFrameInfoStatus.0 as _,
        ) as CFTypeRef;
        if frame_status_ref.is_null() {
            return None;
        }
        let mut frame_status: i64 = 0;
        let result = CFNumberGetValue(
            frame_status_ref as _,
            CFNumberType_kCFNumberSInt64Type,
            mem::transmute(&mut frame_status),
        );
        if result == 0 {
            return None;
        }
        if frame_status != SCFrameStatus_SCFrameStatusComplete {
            return None;
        }
    }

    //let epoch = CMSampleBufferGetPresentationTimeStamp(buffer_ref).epoch;
    let epoch = sample_buffer.sys_ref.get_presentation_timestamp().value;
    let pixel_buffer = CMSampleBufferGetImageBuffer(buffer_ref) as CVPixelBufferRef;

    CVPixelBufferLockBaseAddress(pixel_buffer, 0);

    let width = CVPixelBufferGetWidth(pixel_buffer);
    let height = CVPixelBufferGetHeight(pixel_buffer);
    if width == 0 || height == 0 {
        return None;
    }

    let luminance_bytes_address = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 0);
    let luminance_stride = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 0);
    let luminance_bytes = slice::from_raw_parts(
        luminance_bytes_address as *mut u8,
        height * luminance_stride,
    )
    .to_vec();

    let chrominance_bytes_address = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 1);
    let chrominance_stride = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 1);
    let chrominance_bytes = slice::from_raw_parts(
        chrominance_bytes_address as *mut u8,
        height * chrominance_stride / 2,
    )
    .to_vec();

    CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);

    YUVFrame {
        display_time: epoch as u64,
        width: width as i32,
        height: height as i32,
        luminance_bytes,
        luminance_stride: luminance_stride as i32,
        chrominance_bytes,
        chrominance_stride: chrominance_stride as i32,
    }
    .into()
}

pub unsafe fn create_bgr_frame(sample_buffer: CMSampleBuffer) -> Option<BGRFrame> {
    let buffer_ref = &(*sample_buffer.sys_ref);
    let epoch = sample_buffer.sys_ref.get_presentation_timestamp().value;
    let pixel_buffer = CMSampleBufferGetImageBuffer(buffer_ref) as CVPixelBufferRef;

    CVPixelBufferLockBaseAddress(pixel_buffer, 0);

    let width = CVPixelBufferGetWidth(pixel_buffer);
    let height = CVPixelBufferGetHeight(pixel_buffer);
    if width == 0 || height == 0 {
        return None;
    }

    let base_address = CVPixelBufferGetBaseAddress(pixel_buffer);
    let bytes_per_row = CVPixelBufferGetBytesPerRow(pixel_buffer);

    let data = slice::from_raw_parts(base_address as *mut u8, bytes_per_row * height).to_vec();

    let cropped_data = get_cropped_data(
        data,
        (bytes_per_row / 4) as i32,
        height as i32,
        width as i32,
    );

    CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);

    Some(BGRFrame {
        display_time: epoch as u64,
        width: width as i32, // width does not give accurate results - https://stackoverflow.com/questions/19587185/cvpixelbuffergetbytesperrow-for-cvimagebufferref-returns-unexpected-wrong-valu
        height: height as i32,
        data: remove_alpha_channel(cropped_data),
    })
}

pub unsafe fn create_bgra_frame(sample_buffer: CMSampleBuffer) -> Option<BGRAFrame> {
    let buffer_ref = &(*sample_buffer.sys_ref);
    let epoch = sample_buffer.sys_ref.get_presentation_timestamp().value;
    let pixel_buffer = CMSampleBufferGetImageBuffer(buffer_ref) as CVPixelBufferRef;

    CVPixelBufferLockBaseAddress(pixel_buffer, 0);

    let width = CVPixelBufferGetWidth(pixel_buffer);
    let height = CVPixelBufferGetHeight(pixel_buffer);
    if width == 0 || height == 0 {
        return None;
    }

    let base_address = CVPixelBufferGetBaseAddress(pixel_buffer);
    let bytes_per_row = CVPixelBufferGetBytesPerRow(pixel_buffer);

    let mut data: Vec<u8> = vec![];
    for i in 0..height {
        let start = (base_address as *mut u8).wrapping_add((i * bytes_per_row));
        data.extend_from_slice(slice::from_raw_parts(start, 4 * width));
    }

    CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);

    Some(BGRAFrame {
        display_time: epoch as u64,
        width: width as i32, // width does not give accurate results - https://stackoverflow.com/questions/19587185/cvpixelbuffergetbytesperrow-for-cvimagebufferref-returns-unexpected-wrong-valu
        height: height as i32,
        data,
    })
}

pub unsafe fn create_rgb_frame(sample_buffer: CMSampleBuffer) -> Option<RGBFrame> {
    let buffer_ref = &(*sample_buffer.sys_ref);
    let epoch = sample_buffer.sys_ref.get_presentation_timestamp().value;
    let pixel_buffer = CMSampleBufferGetImageBuffer(buffer_ref) as CVPixelBufferRef;

    CVPixelBufferLockBaseAddress(pixel_buffer, 0);

    let width = CVPixelBufferGetWidth(pixel_buffer);
    let height = CVPixelBufferGetHeight(pixel_buffer);
    if width == 0 || height == 0 {
        return None;
    }

    let base_address = CVPixelBufferGetBaseAddress(pixel_buffer);
    let bytes_per_row = CVPixelBufferGetBytesPerRow(pixel_buffer);

    let data = slice::from_raw_parts(base_address as *mut u8, bytes_per_row * height).to_vec();

    let cropped_data = get_cropped_data(
        data,
        (bytes_per_row / 4) as i32,
        height as i32,
        width as i32,
    );

    CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);

    Some(RGBFrame {
        display_time: epoch as u64,
        width: width as i32, // width does not give accurate results - https://stackoverflow.com/questions/19587185/cvpixelbuffergetbytesperrow-for-cvimagebufferref-returns-unexpected-wrong-valu
        height: height as i32,
        data: convert_bgra_to_rgb(cropped_data),
    })
    // (y_width, y_height, data)
}

pub fn get_output_frame_size(options: &Options) -> [u32; 2] {
    let display = display::get_main_display();
    let display_id = display.display_id;
    let scale = display::get_scale_factor(display_id) as u32;

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

    if output_width % 2 == 1 {
        output_width = output_width - 1;
    }

    if output_height % 2 == 1 {
        output_height = output_height - 1;
    }

    return [output_width, output_height];
}

pub fn get_source_rect(options: &Options) -> CGRect {
    let display = display::get_main_display();
    let width = display.width;
    let height = display.height;

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
        }
        None => CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: width as f64,
                height: height as f64,
            },
        },
    };

    source_rect
}
