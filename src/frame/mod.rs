use std::{mem, slice};

use apple_sys::{CoreMedia::{CFDictionaryGetValue, CFDictionaryRef, CFTypeRef, CFNumberGetValue, CFNumberType_kCFNumberSInt64Type}, ScreenCaptureKit::{SCStreamFrameInfoStatus, SCFrameStatus_SCFrameStatusComplete}};
use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_error_handler::StreamErrorHandler,
    sc_output_handler::{CMSampleBuffer, SCStreamOutputType, StreamOutput},
    sc_shareable_content::SCShareableContent,
    sc_stream::SCStream,
    sc_stream_configuration::SCStreamConfiguration,
    sc_sys::{SCFrameStatus, CMSampleBufferGetPresentationTimeStamp, CMSampleBufferGetImageBuffer, CMSampleBufferGetSampleAttachmentsArray, }, sc_display::SCDisplay,
};
use core_graphics::{
    access::ScreenCaptureAccess,
    display::{CGDirectDisplayID, CGDisplay, CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef, },
};
use core_video_sys::{CVPixelBufferRef, CVPixelBufferLockBaseAddress, CVPixelBufferGetWidth, CVPixelBufferGetHeight, CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferUnlockBaseAddress};

pub struct YUVFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub luminance_bytes: Vec<u8>,
    pub luminance_stride: i32,
    pub chrominance_bytes: Vec<u8>,
    pub chrominance_stride: i32,
}

pub enum FrameData<'a> {
    NV12(&'a YUVFrame),
    BGR0(&'a [u8]),
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
    println!("Epoch: {}", epoch);
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
