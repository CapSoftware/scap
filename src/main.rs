// This program is just a testing application
// Refer to `lib.rs` for the library source code

use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::{Frame, VideoFrame},
};
use std::process;

fn main() {
    // Check if the platform is supported
    if !scap::is_supported() {
        println!("❌ Platform not supported");
        return;
    }

    // Check if we have permission to capture screen
    // If we don't, request it.
    if !scap::has_permission() {
        println!("❌ Permission not granted. Requesting permission...");
        if !scap::request_permission() {
            println!("❌ Permission denied");
            return;
        }
    }

    // // Get recording targets
    // let targets = scap::get_all_targets();

    // Create Options
    let options = Options {
        fps: 60,
        show_cursor: true,
        show_highlight: false,
        excluded_targets: None,
        output_type: scap::frame::FrameType::BGRAFrame,
        output_resolution: scap::capturer::Resolution::_720p,
        crop_area: Some(Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 500.0,
                height: 500.0,
            },
        }),
        captures_audio: true,
        ..Default::default()
    };

    // Create Recorder with options
    let mut recorder = Capturer::build(options).unwrap_or_else(|err| {
        println!("Problem with building Capturer: {err}");
        process::exit(1);
    });

    // Start Capture
    recorder.start_capture();

    // Capture 100 frames
    for i in 0..100 {
        let frame = loop {
            match recorder.get_next_frame().expect("Error") {
                Frame::Video(frame) => {
                    break frame;
                }
                Frame::Audio(_) => {
                    continue;
                }
            }
        };

        match frame {
            VideoFrame::YUVFrame(frame) => {
                println!(
                    "Recieved YUV frame {} of width {} and height {} and pts {:?}",
                    i, frame.width, frame.height, frame.display_time
                );
            }
            VideoFrame::BGR0(frame) => {
                println!(
                    "Received BGR0 frame of width {} and height {}",
                    frame.width, frame.height
                );
            }
            VideoFrame::RGB(frame) => {
                println!(
                    "Recieved RGB frame {} of width {} and height {} and time {:?}",
                    i, frame.width, frame.height, frame.display_time
                );
            }
            VideoFrame::RGBx(frame) => {
                println!(
                    "Recieved RGBx frame of width {} and height {}",
                    frame.width, frame.height
                );
            }
            VideoFrame::XBGR(frame) => {
                println!(
                    "Recieved xRGB frame of width {} and height {}",
                    frame.width, frame.height
                );
            }
            VideoFrame::BGRx(frame) => {
                println!(
                    "Recieved BGRx frame of width {} and height {}",
                    frame.width, frame.height
                );
            }
            VideoFrame::BGRA(frame) => {
                println!(
                    "Recieved BGRA frame {} of width {} and height {} and time {:?}",
                    i, frame.width, frame.height, frame.display_time
                );
            }
        }
    }

    // Stop Capture
    recorder.stop_capture();
}
