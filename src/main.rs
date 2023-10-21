use cypher::{Options, Recorder, Target, TargetKind};

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
    let has_permission = cypher::has_permission();
    if !has_permission {
        println!("❌ Permission not granted");
        return;
    } else {
        println!("✅ Permission granted");
    }

    // let test_options = Options { fps: 32, targets: }

    // #3 Capture the screen (WIP)
    cypher::get_targets();
    let options = Options {
        fps: 32,
        targets: vec![
            Target {
                kind: TargetKind::Window,
                name: String::from("main.rs — circle_area"),
                id: 7780,
            }
        ],
        show_cursor: true,
        show_highlight: true,
    };
    recorder.start_capture(options)
}

