#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

#[derive(Debug)]
pub enum TargetKind {
    Display,
    Window,
    Audio,
}

#[derive(Debug)]
pub struct Target {
    pub kind: TargetKind,
    pub name: String,
    pub id: u32,
}

pub struct Options {
    pub fps: u32,
    pub show_cursor: bool,
    pub show_highlight: bool,
    pub targets: Vec<Target>,
    pub excluded_targets: Option<Vec<Target>>,
}

pub struct Recorder {
    pub id: String,
}

impl Recorder {
    pub fn init() -> Self {
        Recorder {
            id: String::from(""),
        }
    }

    pub fn start_capture(&self, options: Options) {
        #[cfg(target_os = "macos")]
        mac::main();

        #[cfg(target_os = "windows")]
        win::main();
    }

    pub fn stop_capture(&self) {
        // TODO: add stop_capture() to mac and win modules
        // #[cfg(target_os = "macos")]
        // mac::stop_capture();

        // #[cfg(target_os = "windows")]
        // win::stop_capture();
    }
}

pub fn is_supported() -> bool {
    #[cfg(target_os = "macos")]
    let supported = mac::is_supported();

    #[cfg(target_os = "windows")]
    let supported = win::is_supported();

    supported
}

pub fn get_targets() -> Vec<Target> {
    #[cfg(target_os = "macos")]
    let targets = mac::get_targets();

    #[cfg(target_os = "windows")]
    let targets = win::get_targets();

    targets
}

pub fn has_permission() -> bool {
    #[cfg(target_os = "macos")]
    let access = mac::has_permission();

    #[cfg(target_os = "windows")]
    let access = win::has_permission();

    access
}
