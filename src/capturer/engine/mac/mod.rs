use std::sync::mpsc;
use std::{mem, slice};

use screencapturekit::{sc_stream::SCStream, sc_content_filter::{InitParams, SCContentFilter}, sc_stream_configuration::SCStreamConfiguration, sc_error_handler::StreamErrorHandler, sc_output_handler::{StreamOutput, CMSampleBuffer, SCStreamOutputType}, sc_sys::SCFrameStatus};

use crate::frame::{Frame, YUVFrame, FrameType, RGBFrame};
use crate::{capturer::Options, device::display::{self}};
use apple_sys::{CoreMedia::{CFDictionaryGetValue, CFDictionaryRef, CFTypeRef, CFNumberGetValue, CFNumberType_kCFNumberSInt64Type}, ScreenCaptureKit::{SCStreamFrameInfoStatus, SCFrameStatus_SCFrameStatusComplete}};
use screencapturekit::sc_sys::{CMSampleBufferGetImageBuffer, CMSampleBufferGetSampleAttachmentsArray, };
use core_graphics::display::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef, };
use core_video_sys::{CVPixelBufferRef, CVPixelBufferLockBaseAddress, CVPixelBufferGetWidth, CVPixelBufferGetHeight, CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferUnlockBaseAddress, CVPixelBufferGetWidthOfPlane, CVPixelBufferGetHeightOfPlane};

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
                    SCFrameStatus::Complete => {
                        unsafe {
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
                                _ => {
                                    panic!("Unimplemented Output format");
                                }
                            }
                            self.tx.send(frame).unwrap_or(());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

pub fn create_capturer(options: &Options, tx: mpsc::Sender<Frame>) -> SCStream {
    println!("Options: {:?}", options);

    let display = display::get_main_display();
    let display_id = display.display_id;

    let scale = display::get_scale_factor(display_id) as u32;
    let width = display.width * scale;
    let height = display.height * scale;

    let params = InitParams::Display(display);
    let filter = SCContentFilter::new(params);

    let source_rect = options.source_rect.unwrap_or_default();

    let stream_config = SCStreamConfiguration {
        shows_cursor: true,
        width,
        height,
        source_rect,
        ..Default::default()
    };

    let mut stream = SCStream::new(filter, stream_config, ErrorHandler);
    stream.add_output(Capturer::new(tx, options.output_type));

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
    let buffer_ref = &(*sample_buffer.ptr);
    {
        let attachments = CMSampleBufferGetSampleAttachmentsArray(buffer_ref, 0);
        if attachments.is_null() || CFArrayGetCount(attachments as CFArrayRef) == 0 {
            return None;
        }
        let attachment = CFArrayGetValueAtIndex(attachments as CFArrayRef, 0) as CFDictionaryRef;
        let frame_status_ref =
            CFDictionaryGetValue(attachment as CFDictionaryRef, SCStreamFrameInfoStatus.0 as _) as CFTypeRef;
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
    let epoch = sample_buffer.ptr.get_presentation_timestamp().value;
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

pub unsafe fn create_rgb_frame(sample_buffer: CMSampleBuffer) -> Option<RGBFrame> {
    let buffer_ref = &(*sample_buffer.ptr);
    let pixel_buffer = CMSampleBufferGetImageBuffer(buffer_ref) as CVPixelBufferRef;
    // Lock the base address
    CVPixelBufferLockBaseAddress(pixel_buffer, 0);

    // Check the format of the pixel buffer
    // let format = core_video_sys::CVPixelBufferGetPixelFormatType(pixel_buffer);

    // Currently: 875704438, kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange
    // TODO: Capture in BRGA format instead

    // Plane 1 — Y (Luma)
    let y_width = CVPixelBufferGetWidthOfPlane(pixel_buffer, 0);
    let y_height = CVPixelBufferGetHeightOfPlane(pixel_buffer, 0);
    let y_bytes_row = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 0);
    let y_address = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 0);
    let y_stride = y_bytes_row - y_width;

    // Plane 2 — CbCr (Chroma)
    // let c_width = CVPixelBufferGetWidthOfPlane(pixel_buffer, 1);
    let c_height = CVPixelBufferGetHeightOfPlane(pixel_buffer, 1);
    let c_address = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 1);
    let c_bytes_row = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 1);

    let y_data = std::slice::from_raw_parts(
        y_address as *const u8,
        y_height as usize * y_bytes_row as usize,
    );

    let c_data = std::slice::from_raw_parts(
        c_address as *const u8,
        c_height as usize * c_bytes_row as usize,
    );

    // unlock base address
    CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);

    // Logs
    // println!("y_width: {:?}", y_width);
    // println!("y_height: {:?}", y_height);
    // println!("y_address: {:?}", y_address);
    // println!("y_bytes_per_row: {:?}", y_bytes_row);
    // println!("c_width: {:?}", c_width);
    // println!("c_height: {:?}", c_height);
    // println!("c_address: {:?}", c_address);
    // println!("c_bytes_per_row: {:?}", c_bytes_row);

    // println!("y_data: {:?}", y_data);
    // println!("c_data: {:?}", c_data);

    // Convert YUV buffer to RGB
    // let data = Vec::new();
    let data = ycbcr_to_rgb(&y_data, &c_data, y_width, y_height, y_stride);

    Some(RGBFrame {
        width: y_width as i32,
        height: y_height as i32,
        data: data,
    })
    // (y_width, y_height, data)
}
