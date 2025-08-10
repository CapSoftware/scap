use image::RgbaImage;
use scap::{
    capturer::{Area, Capturer, Options, Point, Size},
    frame::{Frame, VideoFrame},
};
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
    // let targets = scap::get_all_targets();
    // println!("Targets: {:?}", targets);

    // All your displays and windows are targets
    // You can filter this and capture the one you need.

    // Create Options
    let options = Options {
        fps: 60,
        target: None, // None captures the primary display
        show_cursor: true,
        show_highlight: true,
        excluded_targets: None,
        output_type: scap::frame::FrameType::BGRAFrame,
        output_resolution: scap::capturer::Resolution::_720p,
        crop_area: Some(Area {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 2000.0,
                height: 1000.0,
            },
        }),
        ..Default::default()
    };

    // Create Capturer
    let mut capturer = Capturer::build(options).unwrap();

    // Create output directory
    let output_dir = "examples/captured_frames";
    if !Path::new(output_dir).exists() {
        fs::create_dir(output_dir).unwrap();
        println!("📁 Created output directory: {}", output_dir);
    }

    // Start Capture
    println!("🎬 Starting capture... Press Enter to stop");
    capturer.start_capture();

    let mut frame_count = 0;
    let start_time = std::time::Instant::now();

    // Create a channel for stopping the capture
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to wait for Enter key
    thread::spawn(move || {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        tx.send(()).unwrap(); // Signal to stop
    });

    // Capture loop - continuously capture frames until Enter is pressed
    loop {
        // Check if user wants to stop (non-blocking)
        if rx.try_recv().is_ok() {
            break; // User pressed Enter
        }

        // Try to get the next frame
        match capturer.get_next_frame() {
            Ok(frame) => {
                frame_count += 1;

                // Save frame as image
                if let Err(e) = save_frame_as_image(&frame, frame_count, output_dir) {
                    eprintln!("Error saving frame {frame_count}: {e}");
                }

                // Print capture info every 30 frames (every 0.5 seconds at 60 FPS)
                if frame_count % 30 == 0 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let fps = frame_count as f64 / elapsed;
                    println!(
                        "📸 Captured frame {frame_count} | FPS: {fps:.1} | Elapsed: {elapsed:.1}s"
                    );
                }
            }
            Err(mpsc::RecvError) => {
                // No frame available yet, continue
                continue;
            }
        }

        // Small delay to prevent excessive CPU usage
        thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    // Stop Capture
    println!("🛑 Stopping capture...");
    capturer.stop_capture();

    let total_time = start_time.elapsed();
    println!("✅ Capture complete!");
    println!("📊 Total frames: {frame_count}");
    println!("⏱️  Total time: {:.2}s", total_time.as_secs_f64());
    println!("📁 Frames saved to: {output_dir}");
}

fn save_frame_as_image(
    frame: &Frame,
    frame_number: u32,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match frame {
        Frame::Video(VideoFrame::BGRA(bgra_frame)) => {
            // Convert BGRA to RGBA (swap red and blue channels)
            let mut rgba_data = Vec::with_capacity(bgra_frame.data.len());
            for chunk in bgra_frame.data.chunks(4) {
                if chunk.len() == 4 {
                    rgba_data.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
                }
            }

            // Create image from RGBA data
            let image =
                RgbaImage::from_raw(bgra_frame.width as u32, bgra_frame.height as u32, rgba_data)
                    .ok_or("Failed to create image")?;

            // Save as PNG
            let filename = format!("{}/frame_{:06}.png", output_dir, frame_number);
            image.save(&filename)?;

            Ok(())
        }
        _ => Err("Unsupported frame type".into()),
    }
}
