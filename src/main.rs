use cypher::{Options, Recorder};

// This program is just a testbed for the library itself
// Refer to the lib.rs file for the actual implementation

fn main() {
    let recorder = Recorder::init();

    // #1 Check if the platform is supported
    let supported = cypher::is_supported();
    if !supported {
        println!("❌ Platform not supported");
        return;
    } else {
        println!("✅ Platform supported");
    }

    // #2 Check if the app has permission to capture the screen

    // macOS Only

    #[cfg(target_os = "macos")]
    {
        let has_permission = cypher::has_permission();
        if !has_permission {
            println!("❌ Permission not granted");
            return;
        } else {
            println!("✅ Permission granted");
        }
    }

    // #3 Capture the screen (WIP)
    cypher::get_targets();
    // recorder.start_capture(options)
}
