use core_graphics::access::ScreenCaptureAccess;
use core_graphics::display::CGMainDisplayID;
use core_video_sys::{
    CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRowOfPlane,
    CVPixelBufferGetHeightOfPlane, CVPixelBufferGetWidthOfPlane, CVPixelBufferLockBaseAddress,
    CVPixelBufferRef, CVPixelBufferUnlockBaseAddress,
};
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

struct ConsoleErrorHandler;

impl sc_error_handler::StreamErrorHandler for ConsoleErrorHandler {
    fn on_error(&self) {
        println!("Error!");
    }
}

struct OutputHandler {}

fn ycbcr_to_rgb(y_data: &[u8], cbcr_data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgb_data = Vec::with_capacity(width * height * 3);

    for j in 0..height {
        for i in 0..width {
            let y_idx = j * width + i;
            let uv_idx = (j / 2) * width + i - i % 2;

            let y = y_data[y_idx] as f32;
            let cb = cbcr_data[uv_idx] as f32 - 128.0;
            let cr = cbcr_data[uv_idx + 1] as f32 - 128.0;

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
                        let pixel_buffer = ptr.get_image_buffer().get_raw() as CVPixelBufferRef;

                        unsafe {
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

                            let rgb_data = ycbcr_to_rgb(&y_data, &c_data, y_bytes_row, y_height);
                            let img = image::RgbImage::from_raw(
                                y_bytes_row as u32,
                                y_height as u32,
                                rgb_data,
                            )
                            .unwrap();

                            // Save the image to disk
                            let filename = format!("frame_{}.png", timestamp);
                            let folder = PathBuf::new().join("test").join(filename);
                            img.save(folder).expect("Failed to save image");

                            // unlock base address
                            CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);
                        }
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
    access.request()
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
