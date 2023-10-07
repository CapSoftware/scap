use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_error_handler,
    sc_output_handler::{CMSampleBuffer, SCStreamOutputType, StreamOutput},
    sc_shareable_content::SCShareableContent,
    sc_stream::SCStream,
    sc_stream_configuration::SCStreamConfiguration,
};

use core_graphics::access::ScreenCaptureAccess;
use core_graphics::display::CGMainDisplayID;

use core_video_sys::{
    CVImageBufferRef, CVPixelBufferGetBaseAddress, CVPixelBufferGetBaseAddressOfPlane,
    CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight, CVPixelBufferGetPixelFormatType,
    CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferUnlockBaseAddress,
};

struct ConsoleErrorHandler;

impl sc_error_handler::StreamErrorHandler for ConsoleErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

struct OutputHandler {}

impl StreamOutput for OutputHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        match of_type {
            SCStreamOutputType::Screen => {
                let frame_status = sample.frame_status;
                println!("Frame Status: {:?}", frame_status);

                let sample_ptr = sample.ptr;

                let timestamp = sample_ptr.get_presentation_timestamp().value;
                let filename = format!("frame_{}.png", timestamp);

                // let pixel_buffer = sample_ptr.get_image_buffer() as CVImageBufferRef;

                // unsafe {
                //     CVPixelBufferLockBaseAddress(pixel_buffer, 0);

                //     let base_address =
                //         CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 0) as *const u8;
                //     let pixel_format_type = CVPixelBufferGetPixelFormatType(pixel_buffer);

                //     // get the pixel buffer's width and height
                //     let width = CVPixelBufferGetWidth(pixel_buffer);
                //     let height = CVPixelBufferGetHeight(pixel_buffer);

                //     let bytes_per_row = CVPixelBufferGetBytesPerRow(pixel_buffer);

                //     // Safe part starts here

                //     CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);
                // }
            }
            _ => {}
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

    // Setup premise
    let content = SCShareableContent::current();

    let main_display_id = unsafe { CGMainDisplayID() };
    let displays = content.displays;

    let main_display = displays
        .iter()
        .find(|display| display.display_id == main_display_id)
        .unwrap_or_else(|| {
            panic!("Main display not found");
        });

    let width = main_display.width;
    let height = main_display.height;

    // Setup screencapturekit
    let params = InitParams::Display(main_display.clone());
    let filter = SCContentFilter::new(params);

    let stream_config = SCStreamConfiguration {
        shows_cursor: true,
        width,
        height,
        ..Default::default()
    };

    let handler = ConsoleErrorHandler;
    let mut stream = SCStream::new(filter, stream_config, handler);

    let output_handler = OutputHandler {
        // video_encoder,
        // video_stream: Arc::clone(&video_stream),
    };

    stream.add_output(output_handler);

    stream.start_capture();
    println!("Capture started. Press Enter to stop.");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    stream.stop_capture();
    println!("Capture stopped.");
}
