use std::sync::mpsc;
use std::{mem, slice};

use screencapturekit::{sc_stream::SCStream, sc_content_filter::{InitParams, SCContentFilter}, sc_display::SCDisplay, sc_stream_configuration::SCStreamConfiguration, sc_error_handler::StreamErrorHandler, sc_output_handler::{StreamOutput, CMSampleBuffer, SCStreamOutputType}, sc_sys::SCFrameStatus};

use crate::frame::{Frame, YUVFrame};
use crate::{capturer::Options, device::display::{self, Display}};
use apple_sys::{CoreMedia::{CFDictionaryGetValue, CFDictionaryRef, CFTypeRef, CFNumberGetValue, CFNumberType_kCFNumberSInt64Type}, ScreenCaptureKit::{SCStreamFrameInfoStatus, SCFrameStatus_SCFrameStatusComplete}};
use screencapturekit::sc_sys::{CMSampleBufferGetImageBuffer, CMSampleBufferGetSampleAttachmentsArray, };
use core_graphics::display::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef, };
use core_video_sys::{CVPixelBufferRef, CVPixelBufferLockBaseAddress, CVPixelBufferGetWidth, CVPixelBufferGetHeight, CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferUnlockBaseAddress};

struct ErrorHandler;
impl StreamErrorHandler for ErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

pub struct Capturer {
    pub tx: mpsc::Sender<Frame>,
}

impl Capturer {
    pub fn new(tx: mpsc::Sender<Frame>) -> Self {
        Capturer { tx }
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
                            if let Some(yuvframe) = create_yuv_frame(sample) {
                                self.tx.send(Frame::YUVFrame(yuvframe)).unwrap_or(());
                            }
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

    let Display::Mac(display) = display::get_main_display();
    let display_id = display.display_id;

    let scale = display::get_scale_factor(display_id) as u32;
    let width = display.width * scale;
    let height = display.height * scale;

    let params = InitParams::Display(display);
    let filter = SCContentFilter::new(params);

    let stream_config = SCStreamConfiguration {
        shows_cursor: true,
        width,
        height,
        ..Default::default()
    };

    let mut stream = SCStream::new(filter, stream_config, ErrorHandler);
    stream.add_output(Capturer::new(tx));

    stream
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