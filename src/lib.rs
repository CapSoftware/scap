#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

mod audio;

#[derive(Debug)]
pub enum TargetKind {
    Display,
    Window,
    Audio,
}

#[derive(Debug)]
pub struct Target {
    pub kind: TargetKind,
    pub title: String,
    pub id: u32,
}

pub struct Options {
    pub fps: u32,
    pub show_cursor: bool,
    pub show_highlight: bool,
    pub targets: Vec<Target>,

    // excluded targets will only work on macOS
    pub excluded_targets: Option<Vec<Target>>,
}

pub struct Recorder {
    audio_recorder: audio::AudioRecorder,

    #[cfg(target_os = "macos")]
    recorder: screencapturekit::sc_stream::SCStream,
}

impl Recorder {
    pub fn init(options: Options) -> Self {
        let audio_recorder = audio::AudioRecorder::new();

        #[cfg(target_os = "macos")]
        let recorder = mac::create_recorder();

        Recorder {
            audio_recorder,
            recorder,
        }
    }

    pub fn start_capture(&mut self) {
        println!("start capture");

        self.audio_recorder.start_recording();

        #[cfg(target_os = "macos")]
        self.recorder.start_capture();

        // #[cfg(target_os = "windows")]
        // win::main();
    }

    pub fn stop_capture(&mut self) {
        println!("stop capture");

        self.audio_recorder.stop_recording();

        #[cfg(target_os = "macos")]
        self.recorder.stop_capture();

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
