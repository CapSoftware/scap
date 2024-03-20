// This program is just a testbed for the library itself
// Refer to the lib.rs file for the actual implementation

use std::path::PathBuf;

use scap::{
    capturer::{CGPoint, CGRect, CGSize, Capturer, Options},
    frame::{Frame, FrameType},
};

fn main() {
    // #1 Check if the platform is supported
    let supported = scap::is_supported();
    if !supported {
        println!("❌ Platform not supported");
        return;
    } else {
        println!("✅ Platform supported");
    }

    // #2 Check if we have permission to capture the screen
    let has_permission = scap::has_permission();
    if !has_permission {
        println!("❌ Permission not granted");
        return;
    } else {
        println!("✅ Permission granted");
    }

    // #3 Get recording targets (WIP)
    let targets = scap::get_targets();
    println!("🎯 Targets: {:?}", targets);

    // #4 Create Options
    let options = Options {
        fps: 60,
        targets,
        show_cursor: true,
        show_highlight: true,
        excluded_targets: None,
        output_type: FrameType::BGRAFrame,
        output_resolution: scap::capturer::Resolution::_720p,
        source_rect: Some(CGRect {
            origin: CGPoint { x: 500.0, y: 50.0 },
            size: CGSize {
                width: 1500.0,
                height: 1000.0,
            },
        }),
        ..Default::default()
    };

    // #5 Create Recorder
    let mut recorder = Capturer::new(options);

    // #6 Start Capture
    recorder.start_capture();

    // #7 Capture 100 frames
    let mut start_time: u64 = 0;
    for i in 0..100 {
        let frame = recorder.get_next_frame().expect("Error");

        match frame {
            Frame::YUVFrame(frame) => {
                println!(
                    "Recieved YUV frame {} of width {} and height {} and pts {}",
                    i, frame.width, frame.height, frame.display_time
                );
            }
            Frame::BGR0(frame) => {
                println!(
                    "Received BGR0 frame of width {} and height {}",
                    frame.width, frame.height
                );
                println!("Frame data => {:?}", frame.data.len());
                let x =
                    image::RgbaImage::from_raw(frame.width as u32, frame.height as u32, frame.data);
                let img = x.unwrap();
                let filename = format!("frame_{}.png", i);
                let folder = PathBuf::new().join("test").join(filename);
                img.save(folder).expect("Failed to save image");
            }
            Frame::RGB(frame) => {
                if (start_time == 0) {
                    start_time = frame.display_time;
                }
                println!(
                    "Recieved RGB frame {} of width {} and height {} and time {}",
                    i,
                    frame.width,
                    frame.height,
                    frame.display_time - start_time
                );
            }
            Frame::RGBx(frame) => {
                println!(
                    "Recieved RGBx frame of width {} and height {}",
                    frame.width, frame.height
                );
                println!("Frame data => {:?}", frame.data.len());
                let x =
                    image::RgbaImage::from_raw(frame.width as u32, frame.height as u32, frame.data);
                let img = x.unwrap();
                let filename = format!("frame_{}.png", i);
                let folder = PathBuf::new().join("test").join(filename);
                img.save(folder).expect("Failed to save image");
            }
            Frame::XBGR(frame) => {
                println!(
                    "Recieved xRGB frame of width {} and height {}",
                    frame.width, frame.height
                );
            }
            Frame::BGRx(frame) => {
                println!(
                    "Recieved BGRx frame of width {} and height {}",
                    frame.width, frame.height
                );
            }
            Frame::BGRA(frame) => {
                if (start_time == 0) {
                    start_time = frame.display_time;
                }
                println!(
                    "Recieved BGRA frame {} of width {} and height {} and time {}",
                    i,
                    frame.width,
                    frame.height,
                    frame.display_time - start_time
                );
            }
        }
    }

    // #8 Stop Capture
    recorder.stop_capture();
}
