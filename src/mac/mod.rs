use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_error_handler,
    sc_output_handler::StreamOutput,
    sc_shareable_content::SCShareableContent,
    sc_stream::{CMSampleBuffer, SCStream},
    sc_stream_configuration::SCStreamConfiguration,
};

use core_video_sys::{
    CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
    CVPixelBufferGetWidth, CVPixelBufferRef,
};

struct ConsoleErrorHandler;

impl sc_error_handler::StreamErrorHandler for ConsoleErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

struct OutputHandler;

impl StreamOutput for OutputHandler {
    fn stream_output(&self, sample: CMSampleBuffer) {
        // println!("Got sample: {:?}", sample);
        let timestamp = sample.presentation_timestamp.value;
        let pixel_buffer =
            sample.pixel_buffer.expect("No buffer found in sample") as CVPixelBufferRef;

        println!("pixel_buffer: {:?}", pixel_buffer);

        // get the pixel buffer's width and height
        let width = unsafe { CVPixelBufferGetWidth(pixel_buffer) };
        let height = unsafe { CVPixelBufferGetHeight(pixel_buffer) };
        // println!("size: {}x{}", width, height);

        // create an ImageBuffer from the pixel buffer data
        let bytes_per_row = unsafe { CVPixelBufferGetBytesPerRow(pixel_buffer) };
        let base_address = unsafe { CVPixelBufferGetBaseAddress(pixel_buffer) };

        // HELP: base_address is 0x0 all the time
        // println!("base_address: {:?}", base_address);

        let buffer = unsafe {
            std::slice::from_raw_parts(base_address as *const u8, bytes_per_row * height)
        };

        // HELP: buffer is empty
        println!("buffer: {:?}", buffer);

        // let image_buffer =
        //     ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, buffer).unwrap();

        // write the ImageBuffer to a PNG file

        // let filename = format!("output-{}.png", timestamp);
        // let file = File::create(&filename).unwrap();
        // let ref mut writer = BufWriter::new(file);
        // let encoder = image::codecs::png::PngEncoder::new(writer);
        // encoder
        //     .encode(
        //         &image_buffer,
        //         width as u32,
        //         height as u32,
        //         image::ColorType::Rgba8,
        //     )
        //     .unwrap();

        // println!("Wrote PNG file: {}", filename);
    }
}

pub fn main() -> Result<(), ()> {
    let content = SCShareableContent::current();
    let display = content.displays.first().unwrap();

    let width = display.width;
    let height = display.height;

    let params = InitParams::Display(display.to_owned());
    let filter = SCContentFilter::new(params);

    let mut stream_config = SCStreamConfiguration::default();

    stream_config.shows_cursor = true;
    stream_config.width = width;
    stream_config.height = height;
    stream_config.captures_audio = true;

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

    Ok(())
}
