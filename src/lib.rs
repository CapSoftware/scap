use rand::Rng;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

pub enum TargetKind {
    Display,
    Window,
    Audio,
}

pub struct Target {
    kind: TargetKind,
    name: String,
    id: u32,
}

pub struct Options {
    fps: u32,
    targets: Vec<Target>,
    show_cursor: bool,
    show_highlight: bool,
}

pub struct Recorder {
    id: String,
}

impl Recorder {
    pub fn init() -> Self {
        Recorder {
            id: String::from(""),
        }
    }

    pub fn start_capture(&self, options: Options) {
        let mut rng = rand::thread_rng();
        let id: u32 = rng.gen();
        println!("id: {}", id);

        #[cfg(target_os = "macos")]
        mac::main();

        #[cfg(target_os = "windows")]
        win::main();
    }

    pub fn stop_capture() {
        #[cfg(target_os = "macos")]
        mac::stop_capture();

        // #[cfg(target_os = "windows")]
        // win::stop_capture();
    }

    pub fn is_supported() -> bool {
        #[cfg(target_os = "macos")]
        let access = mac::is_supported();

        #[cfg(target_os = "windows")]
        let access = win::is_supported();

        access
    }

    pub fn get_targets() -> Vec<Target> {
        #[cfg(target_os = "macos")]
        let targets = mac::get_targets();

        #[cfg(target_os = "windows")]
        let targets = win::get_targets();
        targets
    }

    #[cfg(target_os = "macos")]
    pub fn has_permission() -> bool {
        mac::has_permission()
    }
}
