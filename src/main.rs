// This program is just a testbed for the library itself
// Refer to the lib.rs file for the actual implementation

use scap::{capturer::{Options, Capturer}, frame::Frame};
#[cfg(target_os = "macos")]
use screencapturekit::sc_sys::geometry::{CGRect, CGPoint, CGSize};

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
        output_type: scap::frame::FrameType::YUVFrame,
        #[cfg(target_os = "macos")]
        source_rect: Some(CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize { width: 100.0, height: 100.0 }
        }),
        ..Default::default()
    };

    // #5 Create Recorder
    let mut recorder = Capturer::new(options);

    // #6 Start Capture
    recorder.start_capture();

    // #7 Capture 100 frames
    for _ in 0..100 {
        let frame = recorder.get_next_frame().expect("Error");
        match frame {
            Frame::YUVFrame(frame) => {
                println!("{}", frame.display_time)
            }
            Frame::BGR0(_) => {
                println!("Recvd windows frame");
            }
            Frame::RGB(frame) => {
                println!("Recieved frame of width {} and height {}", frame.width, frame.height);
            }
        }
    }

    // #8 Stop Capture
    recorder.stop_capture();
}
