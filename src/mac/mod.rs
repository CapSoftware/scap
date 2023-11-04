use core_graphics::{
    access::ScreenCaptureAccess,
    display::{CGDirectDisplayID, CGDisplay},
};
use core_video_sys::CVPixelBufferRef;
use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_error_handler,
    sc_output_handler::{CMSampleBuffer, SCStreamOutputType, StreamOutput},
    sc_shareable_content::SCShareableContent,
    sc_stream::SCStream,
    sc_stream_configuration::SCStreamConfiguration,
    sc_sys::SCFrameStatus,
};
use std::{path::PathBuf, process::Command};

use crate::{Target, TargetKind};

mod temp;

struct ConsoleErrorHandler;

impl sc_error_handler::StreamErrorHandler for ConsoleErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

struct OutputHandler {}

// Get the scale factor of given display
fn get_scale_factor(display_id: CGDirectDisplayID) -> u64 {
    let mode = CGDisplay::new(display_id).display_mode().unwrap();
    let width = mode.width();
    let pixel_width = mode.pixel_width();
    pixel_width / width
}

impl StreamOutput for OutputHandler {
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
                        let timestamp = ptr.get_presentation_timestamp().value;
                        let buffer = ptr.get_image_buffer().get_raw() as CVPixelBufferRef;

                        let (width, height, data) = unsafe { temp::get_data_from_buffer(buffer) };

                        let img =
                            image::RgbImage::from_raw(width as u32, height as u32, data).unwrap();

                        // Save the image to disk
                        let filename = format!("frame_{}.png", timestamp);
                        let folder = PathBuf::new().join("test").join(filename);
                        img.save(folder).expect("Failed to save image");
                    }
                }
            }
            SCStreamOutputType::Audio => {
                // TODO: Handle audios
            }
        }
    }
}

pub fn main() {
    let display = temp::get_main_display();
    let display_id = display.display_id;

    let scale_factor = get_scale_factor(display_id) as u32;

    let width = display.width * scale_factor;
    let height = display.height * scale_factor;

    // Setup screencapturekit
    let params = InitParams::Display(display.to_owned());
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
    access.request()
}

pub fn is_supported() -> bool {
    /* 
     Checks the product os version from the sw_vers
     Returns true if the product version is greater than min_version
    */

    // min_version is vec<u8> form
    let min_version: Vec<u8> = "12.3\n".as_bytes().to_vec();

    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .expect("Failed to execute sw_vers command");

    // current os version received in vec<u8> format
    let os_version = output.stdout;

    os_version >= min_version

}

pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    let content = SCShareableContent::current();
    let displays = content.displays;

    for display in displays {
        // println!("Display: {:?}", display);
        let name = format!("Display {}", display.display_id); // TODO: get this from core-graphics

        let target = Target {
            kind: TargetKind::Display,
            id: display.display_id,
            name,
        };

        targets.push(target);
    }

    // TODO: finish adding windows
    // let windows = content.windows;
    // for window in windows {
    //     match window.title {
    //         Some(title) => {
    //             let name = title;
    //             let app = window.owning_application.unwrap().application_name.unwrap();
    //             println!("Title: {:?}", app);

    //             let target = Target {
    //                 kind: TargetKind::Window,
    //                 id: window.window_id,
    //                 name,
    //             };

    //             targets.push(target);
    //         }
    //         None => {}
    //     }
    // }

    // println!("Targets: {:?}", targets);
    targets
}
