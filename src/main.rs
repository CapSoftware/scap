use cypher::{Options, Recorder};

// This program is just a testbed for the library itself
// Refer to the lib.rs file for the actual implementation

fn main() {
    // #1 Check if the platform is supported
    let supported = cypher::is_supported();
    if !supported {
        println!("❌ Platform not supported");
        return;
    } else {
        println!("✅ Platform supported");
    }

    // #2 Check if we have permission to capture the screen
    let has_permission = cypher::has_permission();
    if !has_permission {
        println!("❌ Permission not granted");
        return;
    } else {
        println!("✅ Permission granted");
    }

    // #3 Get recording targets (WIP)
    let targets = cypher::get_targets();
    println!("🎯 Targets: {:?}", targets);

    // #4 Capture the screen (WIP)
    // let recorder = Recorder::init();
}
