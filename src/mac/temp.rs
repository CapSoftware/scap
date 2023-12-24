use core_graphics::display::CGMainDisplayID;
use core_video_sys::{
    CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRowOfPlane,
    CVPixelBufferGetHeightOfPlane, CVPixelBufferGetWidthOfPlane, CVPixelBufferLockBaseAddress,
    CVPixelBufferRef, CVPixelBufferUnlockBaseAddress,
};
use screencapturekit::sc_display::SCDisplay;
use screencapturekit::sc_shareable_content::SCShareableContent;

// Convert YCbCr to RGB
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

// TEMP: get main display
pub fn get_main_display() -> SCDisplay {
    let content = SCShareableContent::current();
    let displays = content.displays;

    let main_display_id = unsafe { CGMainDisplayID() };
    let main_display = displays
        .iter()
        .find(|display| display.display_id == main_display_id)
        .unwrap_or_else(|| {
            panic!("Main display not found");
        });

    main_display.to_owned()
}

// TEMP: get rgb data from sample buffer
pub unsafe fn get_data_from_buffer(pixel_buffer: CVPixelBufferRef) -> (usize, usize, Vec<u8>) {
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

    (y_width, y_height, data)
}
