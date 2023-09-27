use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_error_handler,
    sc_output_handler::StreamOutput,
    sc_shareable_content::SCShareableContent,
    sc_stream::{CMSampleBuffer, SCStream},
    sc_stream_configuration::SCStreamConfiguration,
};

use core_video_sys::CVPixelBufferRef;

struct ConsoleErrorHandler;

impl sc_error_handler::StreamErrorHandler for ConsoleErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

struct OutputHandler;

impl StreamOutput for OutputHandler {
    fn stream_output(&self, sample: CMSampleBuffer) {
        println!("Got sample!");

        let time_cmtime = sample.presentation_timestamp;
        let time = time_cmtime.value;
        println!("Time: {:?}", time);

        let pixel_buffer_ref = sample.pixel_buffer.unwrap();
        println!("PixelBufferRef: {:?}", pixel_buffer_ref);

        // let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer)
    }
}

pub fn main() -> Result<(), ()> {
    let content = SCShareableContent::current();
    // contains displays, applications and windows

    let display = content.displays.first().unwrap();

    let width = display.width;
    let height = display.height;

    let params = InitParams::Display(display.clone());
    let filter = SCContentFilter::new(params);

    let mut stream_config = SCStreamConfiguration::default();

    stream_config.shows_cursor = true;
    stream_config.width = width;
    stream_config.height = height;
    stream_config.captures_audio = false;
    stream_config.scales_to_fit = true;

    // println!("Stream Config: {:?}", stream_config);

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
