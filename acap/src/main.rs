use acap::capturer::Capturer;

fn main() {
    let mut recorder = Capturer::new();
    recorder.start_capture();

    // Capture audio for 10 frames
    for _ in 0..50 {
        let frame = recorder.get_next_frame().expect("Error");
        println!("Data of length {:?} received", frame.data.len());
    }

    recorder.stop_capture();
}