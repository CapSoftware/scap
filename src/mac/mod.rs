use core_graphics::access::ScreenCaptureAccess;
use core_graphics::display::CGMainDisplayID;
use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_error_handler,
    sc_output_handler::{CMSampleBuffer, SCStreamOutputType, StreamOutput},
    sc_shareable_content::SCShareableContent,
    sc_stream::SCStream,
    sc_stream_configuration::SCStreamConfiguration,
    sc_sys::{CMSampleBufferGetImageBuffer, SCFrameStatus},
};
use std::process::Command;

struct ConsoleErrorHandler;

impl sc_error_handler::StreamErrorHandler for ConsoleErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

struct OutputHandler {}

impl StreamOutput for OutputHandler {
    // CMSampleBuffer
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        match of_type {
            SCStreamOutputType::Screen => {
                let frame_status = &sample.frame_status;

                match frame_status {
                    SCFrameStatus::Idle => {
                        return;
                    }
                    _ => {
                        let ptr = sample.ptr;
                        println!("Id<CMSampleBufferRef>: {:?}", ptr);

                        // error on the following line
                        // let owned = *ptr;

                        // But this command needs to own CMSampleBufferRef
                        // let image_buffer = unsafe { CMSampleBufferGetImageBuffer(&owned) };
                        // println!("CMSampleBufferRef: {:?}", image_buffer);
                    }
                }
            }
            SCStreamOutputType::Audio => {
                // TODO: Handle audio
            }
        }
    }
}

pub fn main() {
    let content = SCShareableContent::current();
    let displays = content.displays;

    let main_display_id = unsafe { CGMainDisplayID() };
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

    let error_handler = ConsoleErrorHandler;
    let mut stream = SCStream::new(filter, stream_config, error_handler);
    let output_handler = OutputHandler {};
    stream.add_output(output_handler);

    stream.start_capture();
    println!("Capture started. Press Enter to stop.");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    stream.stop_capture();
    println!("Capture stopped.");
}

pub fn has_permission() -> bool {
    let access = ScreenCaptureAccess::default();
    access.preflight()
}

pub fn is_supported() -> bool {
    let min_suported_macos_version = 12.3;

    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .expect("Failed to execute sw_vers command");

    let os_version = String::from_utf8(output.stdout).expect("Output not UTF-8");
    let os_version = os_version.trim().parse::<f64>().unwrap();

    if os_version < min_suported_macos_version {
        println!("macOS version {} is not supported", os_version);
        return false;
    } else {
        return true;
    }
}

pub fn get_targets() {
    let content = SCShareableContent::current();
    let displays = content.displays;

    for display in displays {
        println!("Display: {:?}", display);
    }
}
