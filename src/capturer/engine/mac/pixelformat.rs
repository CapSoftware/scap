use std::time::SystemTime;

use screencapturekit::output::{
    sc_stream_frame_info::{SCFrameStatus, SCStreamFrameInfo},
    CMSampleBuffer, LockTrait,
};

use super::{apple_sys::CMTimeGetSeconds, pixel_buffer::get_sample_buffer_pts};
use crate::frame::{
    convert_bgra_to_rgb, get_cropped_data, remove_alpha_channel, BGRAFrame, BGRFrame, RGBFrame,
    YUVFrame,
};

// Returns a frame's presentation timestamp in nanoseconds since an arbitrary start time.
// This is typically yielded from a monotonic clock started on system boot.
pub fn get_pts_in_nanoseconds(sample_buffer: &CMSampleBuffer) -> u64 {
    let pts = get_sample_buffer_pts(sample_buffer);

    let seconds = unsafe { CMTimeGetSeconds(pts) };

    (seconds * 1_000_000_000.).trunc() as u64
}

pub unsafe fn create_yuv_frame(
    sample_buffer: CMSampleBuffer,
    display_time: SystemTime,
) -> Option<YUVFrame> {
    let info = SCStreamFrameInfo::from_sample_buffer(&sample_buffer).unwrap();
    let status = info.status();
    if !matches!(status, SCFrameStatus::Complete) {
        return None;
    }

    let pixel_buffer = sample_buffer.get_pixel_buffer().unwrap();
    let bytes = pixel_buffer.lock().unwrap();
    let width = pixel_buffer.get_width();
    let height = pixel_buffer.get_height();

    if width == 0 || height == 0 {
        return None;
    }

    let luminance_bytes = bytes.as_slice_plane(0).to_vec();
    let luminance_stride = pixel_buffer.get_bytes_per_row_of_plane(0);
    let chrominance_bytes = bytes.as_slice_plane(1).to_vec();
    let chrominance_stride = pixel_buffer.get_bytes_per_row_of_plane(1);

    YUVFrame {
        display_time,
        width: width as i32,
        height: height as i32,
        luminance_bytes,
        luminance_stride: luminance_stride as i32,
        chrominance_bytes,
        chrominance_stride: chrominance_stride as i32,
    }
    .into()
}

pub unsafe fn create_bgr_frame(
    sample_buffer: CMSampleBuffer,
    display_time: SystemTime,
) -> Option<BGRFrame> {
    let pixel_buffer = sample_buffer.get_pixel_buffer().unwrap();
    let bytes = pixel_buffer.lock().unwrap();
    let width = pixel_buffer.get_width();
    let height = pixel_buffer.get_height();

    if width == 0 || height == 0 {
        return None;
    }

    let bytes_per_row = pixel_buffer.get_bytes_per_row();
    let data = bytes.to_vec();

    let cropped_data = get_cropped_data(
        data,
        (bytes_per_row / 4) as i32,
        height as i32,
        width as i32,
    );

    Some(BGRFrame {
        display_time,
        width: width as i32, // width does not give accurate results - https://stackoverflow.com/questions/19587185/cvpixelbuffergetbytesperrow-for-cvimagebufferref-returns-unexpected-wrong-valu
        height: height as i32,
        data: remove_alpha_channel(cropped_data),
    })
}

pub unsafe fn create_bgra_frame(
    sample_buffer: CMSampleBuffer,
    display_time: SystemTime,
) -> Option<BGRAFrame> {
    let pixel_buffer = sample_buffer.get_pixel_buffer().unwrap();
    let bytes = pixel_buffer.lock().unwrap();
    let width = pixel_buffer.get_width();
    let height = pixel_buffer.get_height();

    if width == 0 || height == 0 {
        return None;
    }

    let bytes_per_row = pixel_buffer.get_bytes_per_row();

    let mut data: Vec<u8> = vec![];

    for i in 0..height {
        let base = i * bytes_per_row;
        data.extend_from_slice(&bytes[base as usize..(base + 4 * width) as usize]);
    }

    Some(BGRAFrame {
        display_time,
        width: width as i32, // width does not give accurate results - https://stackoverflow.com/questions/19587185/cvpixelbuffergetbytesperrow-for-cvimagebufferref-returns-unexpected-wrong-valu
        height: height as i32,
        data,
    })
}

pub unsafe fn create_rgb_frame(
    sample_buffer: CMSampleBuffer,
    display_time: SystemTime,
) -> Option<RGBFrame> {
    let pixel_buffer = sample_buffer.get_pixel_buffer().unwrap();
    let bytes = pixel_buffer.lock().unwrap();
    let width = pixel_buffer.get_width();
    let height = pixel_buffer.get_height();

    if width == 0 || height == 0 {
        return None;
    }

    let bytes_per_row = pixel_buffer.get_bytes_per_row();
    let data = bytes.to_vec();

    let cropped_data = get_cropped_data(
        data,
        (bytes_per_row / 4) as i32,
        height as i32,
        width as i32,
    );

    Some(RGBFrame {
        display_time,
        width: width as i32, // width does not give accurate results - https://stackoverflow.com/questions/19587185/cvpixelbuffergetbytesperrow-for-cvimagebufferref-returns-unexpected-wrong-valu
        height: height as i32,
        data: convert_bgra_to_rgb(cropped_data),
    })
}
