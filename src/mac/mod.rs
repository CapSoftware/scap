use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_error_handler,
    sc_output_handler::StreamOutput,
    sc_shareable_content::SCShareableContent,
    sc_stream::{CMSampleBuffer, SCStream},
    sc_stream_configuration::{PixelFormat, SCStreamConfiguration},
};

use core_graphics::{access::ScreenCaptureAccess, display::CGMainDisplayID};

use core_video_sys::{
    CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
    CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress,
    CVPixelBufferRef, CVPixelBufferUnlockBaseAddress,
};

use image::{ImageBuffer, Rgba};
use std::fs::File;
use std::io::BufWriter;

struct ConsoleErrorHandler;

impl sc_error_handler::StreamErrorHandler for ConsoleErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

struct OutputHandler;

impl StreamOutput for OutputHandler {
    fn stream_output(&self, sample: CMSampleBuffer) {
        let timestamp = sample.presentation_timestamp.value;
        let pixel_buffer = sample.pixel_buffer.unwrap() as CVPixelBufferRef;

        unsafe {
            CVPixelBufferLockBaseAddress(pixel_buffer, 0);
        }

        let base_address = unsafe { CVPixelBufferGetBaseAddress(pixel_buffer) };

        let pixel_format_type = unsafe { CVPixelBufferGetPixelFormatType(pixel_buffer) };

        println!("Pixel format type: {}", pixel_format_type);

        // get the pixel buffer's width and height
        let width = unsafe { CVPixelBufferGetWidth(pixel_buffer) };
        let height = unsafe { CVPixelBufferGetHeight(pixel_buffer) };
        let bytes_per_row = unsafe { CVPixelBufferGetBytesPerRow(pixel_buffer) };
        println!("Width: {}", width);
        println!("Height: {}", height);
        println!("Bytes per row: {}", bytes_per_row);

        let buffer_size = (height as usize) * bytes_per_row as usize;
        println!("Expected buffer size: {}", buffer_size);

        let buffer_data = unsafe {
            std::slice::from_raw_parts(
                base_address as *const u8,
                (height as usize) * bytes_per_row as usize,
            )
        };

        // Create an empty image buffer
        let mut img = ImageBuffer::new(width as u32, height as u32);

        for y in 0..height {
            for x in 0..width {
                let offset = (y * bytes_per_row + x) as usize;
                if offset + 3 < buffer_data.len() {
                    if y < 30 && x < 10 {
                        // Adjust these values to control how many pixels you log
                        println!(
                            "Pixel at ({}, {}): R = {}, G = {}, B = {}, A = {}",
                            x,
                            y,
                            buffer_data[offset + 2],
                            buffer_data[offset + 1],
                            buffer_data[offset + 0],
                            buffer_data[offset + 3]
                        );
                    }

                    let pixel = Rgba([
                        buffer_data[offset + 2], // Red
                        buffer_data[offset + 1], // Green
                        buffer_data[offset + 0], // Blue
                        // buffer_data[offset + 3], // Alpha
                        255,
                    ]);
                    img.put_pixel(x as u32, y as u32, pixel);
                } else {
                    println!("Skipping pixel at x={}, y={} due to out of bounds", x, y);
                }
            }
        }

        // Save the image to disk
        // let filename = format!("frame_{}.png", timestamp);
        // img.save(filename).expect("Failed to save image");

        unsafe {
            CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);
        }
    }
}

pub fn main() {
    // check for screen capture permission
    let access = ScreenCaptureAccess::default();
    let access = access.preflight();

    // if access isnt true, log it and return
    if !access {
        println!("screencapture access not granted");
        return;
    }

    let content = SCShareableContent::current();

    let main_display_id = unsafe { CGMainDisplayID() };
    let displays = content.displays;

    // find the display with the same id as the main display
    let main_display = displays
        .iter()
        .find(|display| display.display_id == main_display_id)
        .unwrap_or_else(|| {
            panic!("Main display not found");
        });

    // println!("main_display: {:?}", main_display);
    let width = main_display.width;
    let height = main_display.height;

    // let params = InitParams::DesktopIndependentWindow(*texts_window);
    let params = InitParams::Display(main_display.clone());
    let filter = SCContentFilter::new(params);

    let mut stream_config = SCStreamConfiguration::default();

    stream_config.shows_cursor = true;
    stream_config.width = width;
    stream_config.height = height;
    stream_config.pixel_format = PixelFormat::ARGB8888;
    stream_config.queue_depth = 6;
    stream_config.color_space_name = "sRGB";

    let handler = ConsoleErrorHandler;
    let mut stream = SCStream::new(filter, stream_config, handler);

    let output_handler = OutputHandler;
    stream.add_output(output_handler);

    stream.start_capture();
    println!("Capture started. Press Enter to stop.");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    stream.stop_capture();
    println!("Capture stopped.");
}
