// This program is just a testing application
// Refer to `lib.rs` for the library source code

use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::Frame,
    get_all_targets, get_main_display, Target,
};

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

    // Get recording targets
    let targets = scap::get_all_targets();
    let windows: Vec<Target> = targets
        .into_iter()
        .filter(|target| matches!(target, Target::Window(_)))
        .collect();
    println!("windows: {:?}", windows);
    let window = windows
        .iter()
        .find(|target| {
            if let Target::Window(window) = target {
                window.title == "Cursor"
            } else {
                false
            }
        })
        .expect("No window found");

    // Create Options
    let options = Options {
        fps: 60,
        show_cursor: true,
        show_highlight: true,
        excluded_targets: None,
        target: Some(window.clone()),
        output_type: scap::frame::FrameType::BGRAFrame,
        output_resolution: scap::capturer::Resolution::_720p,
        crop_area: Some(Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 500.0,
                height: 500.0,
            },
        }),
        ..Default::default()
    };

    // Create Recorder with options
    let mut recorder = Capturer::new(options);

    // Start Capture
    recorder.start_capture();

    // Capture 100 frames
    let mut start_time: u64 = 0;
    for i in 0..100 {
        let frame = recorder.get_next_frame().expect("Error");

        match frame {
            Frame::YUVFrame(frame, metadata) => {
                println!(
                    "Recieved YUV frame {} of width {} and height {} and pts {}",
                    i, frame.width, frame.height, frame.display_time
                );
                println!("App name: {:?}", metadata.app_name);
                println!("Window name: {:?}", metadata.window_name);
            }
            Frame::BGR0(frame, metadata) => {
                println!(
                    "Received BGR0 frame of width {} and height {}",
                    frame.width, frame.height
                );
                println!("App name: {:?}", metadata.app_name);
                println!("Window name: {:?}", metadata.window_name);
            }
            Frame::RGB(frame, metadata) => {
                if start_time == 0 {
                    start_time = frame.display_time;
                }
                println!(
                    "Recieved RGB frame {} of width {} and height {} and time {}",
                    i,
                    frame.width,
                    frame.height,
                    frame.display_time - start_time
                );
                println!("App name: {:?}", metadata.app_name);
                println!("Window name: {:?}", metadata.window_name);
            }
            Frame::RGBx(frame, metadata) => {
                println!(
                    "Recieved RGBx frame of width {} and height {}",
                    frame.width, frame.height
                );
                println!("App name: {:?}", metadata.app_name);
                println!("Window name: {:?}", metadata.window_name);
            }
            Frame::XBGR(frame, metadata) => {
                println!(
                    "Recieved xRGB frame of width {} and height {}",
                    frame.width, frame.height
                );
                println!("App name: {:?}", metadata.app_name);
                println!("Window name: {:?}", metadata.window_name);
            }
            Frame::BGRx(frame, metadata) => {
                println!(
                    "Recieved BGRx frame of width {} and height {}",
                    frame.width, frame.height
                );
                println!("App name: {:?}", metadata.app_name);
                println!("Window name: {:?}", metadata.window_name);
            }
            Frame::BGRA(frame, metadata) => {
                if start_time == 0 {
                    start_time = frame.display_time;
                }
                println!(
                    "Recieved BGRA frame {} of width {} and height {} and time {}",
                    i,
                    frame.width,
                    frame.height,
                    frame.display_time - start_time
                );
                println!("App name: {:?}", metadata.app_name);
                println!("Window name: {:?}", metadata.window_name);
            }
        }
    }

    // Stop Capture
    recorder.stop_capture();
}
