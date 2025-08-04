use std::time::SystemTime;

use cidre::{cm, cv};

use crate::frame::{
    convert_bgra_to_rgb, get_cropped_data, remove_alpha_channel, BGRAFrame, BGRFrame, RGBFrame,
    YUVFrame,
};

pub unsafe fn create_yuv_frame(
    sample_buffer: &mut cm::SampleBuf,
    display_time: SystemTime,
) -> Option<YUVFrame> {
    let image_buffer = sample_buffer.image_buf_mut().unwrap();

    unsafe {
        image_buffer
            .lock_base_addr(cv::pixel_buffer::LockFlags::DEFAULT)
            .result()
            .unwrap()
    };

    let width = image_buffer.width();
    let height = image_buffer.height();

    if width == 0 || height == 0 {
        return None;
    }

    let luminance_stride = image_buffer.plane_bytes_per_row(0);
    let luminance_bytes = unsafe {
        std::slice::from_raw_parts(
            image_buffer.plane_base_address(0),
            luminance_stride * image_buffer.plane_height(0),
        )
    }
    .to_vec();

    let chrominance_stride = image_buffer.plane_bytes_per_row(0);
    let chrominance_bytes = unsafe {
        std::slice::from_raw_parts(
            image_buffer.plane_base_address(0),
            luminance_stride * image_buffer.plane_height(0),
        )
    }
    .to_vec();

    unsafe {
        image_buffer
            .unlock_lock_base_addr(cv::pixel_buffer::LockFlags::DEFAULT)
            .result()
            .unwrap()
    };

    Some(YUVFrame {
        display_time,
        width: width as i32,
        height: height as i32,
        luminance_bytes,
        luminance_stride: luminance_stride as i32,
        chrominance_bytes,
        chrominance_stride: chrominance_stride as i32,
    })
}

pub unsafe fn create_bgr_frame(
    sample_buffer: &mut cm::SampleBuf,
    display_time: SystemTime,
) -> Option<BGRFrame> {
    let image_buffer = sample_buffer.image_buf_mut().unwrap();

    unsafe {
        image_buffer
            .lock_base_addr(cv::pixel_buffer::LockFlags::DEFAULT)
            .result()
            .unwrap()
    };

    let width = image_buffer.width();
    let height = image_buffer.height();

    if width == 0 || height == 0 {
        return None;
    }

    let stride = image_buffer.plane_bytes_per_row(0);
    let bytes = unsafe {
        std::slice::from_raw_parts(
            image_buffer.plane_base_address(0),
            stride * image_buffer.plane_height(0),
        )
    }
    .to_vec();

    let cropped_data = get_cropped_data(bytes, (stride / 4) as i32, height as i32, width as i32);

    Some(BGRFrame {
        display_time,
        width: width as i32, // width does not give accurate results - https://stackoverflow.com/questions/19587185/cvpixelbuffergetbytesperrow-for-cvimagebufferref-returns-unexpected-wrong-valu
        height: height as i32,
        data: remove_alpha_channel(cropped_data),
    })
}

pub unsafe fn create_bgra_frame(
    sample_buffer: &mut cm::SampleBuf,
    display_time: SystemTime,
) -> Option<BGRAFrame> {
    let image_buffer = sample_buffer.image_buf_mut().unwrap();

    unsafe {
        image_buffer
            .lock_base_addr(cv::pixel_buffer::LockFlags::DEFAULT)
            .result()
            .unwrap()
    };

    let width = image_buffer.width();
    let height = image_buffer.height();

    if width == 0 || height == 0 {
        return None;
    }

    let stride = image_buffer.plane_bytes_per_row(0);

    let mut data: Vec<u8> = vec![];

    let bytes = unsafe {
        std::slice::from_raw_parts(
            image_buffer.plane_base_address(0),
            stride * image_buffer.plane_height(0),
        )
    };

    for i in 0..height {
        let base = i * stride;
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
    sample_buffer: &mut cm::SampleBuf,
    display_time: SystemTime,
) -> Option<RGBFrame> {
    let image_buffer = sample_buffer.image_buf_mut().unwrap();

    unsafe {
        image_buffer
            .lock_base_addr(cv::pixel_buffer::LockFlags::DEFAULT)
            .result()
            .unwrap()
    };

    let width = image_buffer.width();
    let height = image_buffer.height();

    if width == 0 || height == 0 {
        return None;
    }

    let stride = image_buffer.plane_bytes_per_row(0);

    let bytes = unsafe {
        std::slice::from_raw_parts(
            image_buffer.plane_base_address(0),
            stride * image_buffer.plane_height(0),
        )
    }
    .to_vec();

    let cropped_data = get_cropped_data(bytes, (stride / 4) as i32, height as i32, width as i32);

    Some(RGBFrame {
        display_time,
        width: width as i32, // width does not give accurate results - https://stackoverflow.com/questions/19587185/cvpixelbuffergetbytesperrow-for-cvimagebufferref-returns-unexpected-wrong-valu
        height: height as i32,
        data: convert_bgra_to_rgb(cropped_data),
    })
}
