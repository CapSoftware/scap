use crate::{Target, TargetKind};
use core_graphics::{
    access::ScreenCaptureAccess,
    display::{CGDirectDisplayID, CGDisplay},
};
use core_video_sys::CVPixelBufferRef;
use screencapturekit::{
    sc_content_filter::{InitParams, SCContentFilter},
    sc_error_handler::StreamErrorHandler,
    sc_output_handler::{CMSampleBuffer, SCStreamOutputType, StreamOutput},
    sc_shareable_content::SCShareableContent,
    sc_stream::SCStream,
    sc_stream_configuration::SCStreamConfiguration,
    sc_sys::SCFrameStatus,
};
use std::process::Command;

mod temp;
struct ErrorHandler;

impl StreamErrorHandler for ErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

fn get_scale_factor(display_id: CGDirectDisplayID) -> u64 {
    let mode = CGDisplay::new(display_id).display_mode().unwrap();
    mode.pixel_width() / mode.width()
}

struct Capturer {}

impl StreamOutput for Capturer {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        match of_type {
            SCStreamOutputType::Screen => {
                let frame_status = &sample.frame_status;

                match frame_status {
                    SCFrameStatus::Complete => {
                        let ptr = sample.ptr;
                        let timestamp = ptr.get_presentation_timestamp().value;
                        let buffer = ptr.get_image_buffer().get_raw() as CVPixelBufferRef;

                        let (width, height, data) = unsafe { temp::get_data_from_buffer(buffer) };

                        println!("Frame: {}", timestamp);

                        // FOR TESTING ONLY
                        // Create an image and save frame to disk
                        // let x = image::RgbImage::from_raw(width as u32, height as u32, data);
                        // let img = x.unwrap();

                        // let filename = format!("frame_{}.png", timestamp);
                        // let folder = PathBuf::new().join("test").join(filename);
                        // img.save(folder).expect("Failed to save image");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

pub fn create_recorder() -> SCStream {
    let display = temp::get_main_display();
    let display_id = display.display_id;

    let scale = get_scale_factor(display_id) as u32;
    let width = display.width * scale;
    let height = display.height * scale;

    let params = InitParams::Display(display.to_owned());
    let filter = SCContentFilter::new(params);

    let stream_config = SCStreamConfiguration {
        shows_cursor: true,
        width,
        height,
        ..Default::default()
    };

    let mut stream = SCStream::new(filter, stream_config, ErrorHandler);
    stream.add_output(Capturer {});

    stream
}

pub fn has_permission() -> bool {
    let access = ScreenCaptureAccess::default();
    access.request()
}

pub fn is_supported() -> bool {
    let min_version: Vec<u8> = "12.3\n".as_bytes().to_vec();
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .expect("Failed to execute sw_vers command");

    let os_version = output.stdout;

    os_version >= min_version
}

pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    let content = SCShareableContent::current();
    let displays = content.displays;

    for display in displays {
        // println!("Display: {:?}", display);
        let title = format!("Display {}", display.display_id); // TODO: get this from core-graphics

        let target = Target {
            kind: TargetKind::Display,
            id: display.display_id,
            title,
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
