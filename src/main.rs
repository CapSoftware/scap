use cypher::{Options, Recorder};

// This program is just a testbed for the library itself
// Refer to the lib.rs file for the actual implementation

fn main() {
    let recorder = Recorder::init();

    // #1 Check if the platform is supported
    let supported = recorder.is_supported();
    if !supported {
        println!("❌ Platform not supported");
        return;
    } else {
        println!("✅ Platform supported");
    }

    // #2 Check if the app has permission to capture the screen
    let has_permission = recorder.has_permission();
    if !has_permission {
        println!("❌ Permission not granted");
        return;
    } else {
        println!("✅ Permission granted");
    }

    // #3 Capture the screen (WIP)
    recorder.start_capture(options)
}
