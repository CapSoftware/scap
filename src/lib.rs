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
    options: Options,

    #[cfg(target_os = "macos")]
    recorder: screencapturekit::sc_stream::SCStream,

    #[cfg(target_os = "windows")]
    recorder: Option<windows_capture::capture::CaptureControl>,
}

impl Recorder {
    pub fn init(options: Options) -> Self {
        let audio_recorder = audio::AudioRecorder::new();

        #[cfg(target_os = "macos")]
        let recorder = mac::create_recorder();

        #[cfg(target_os = "windows")]
        let recorder = None;

        Recorder {
            audio_recorder,
            recorder,
            options,
        }
    }

    pub fn start_recording(&mut self) {
        self.audio_recorder.start_recording();

        #[cfg(target_os = "macos")]
        self.recorder.start_capture();

        #[cfg(target_os = "windows")]
        {
            let recorder = win::create_recorder();
            self.recorder = Some(recorder);
        }
    }

    pub fn stop_recording(&mut self) {
        self.audio_recorder.stop_recording();

        #[cfg(target_os = "macos")]
        self.recorder.stop_capture();

        // TODO: temporary workaround until I find a better way
        #[cfg(target_os = "windows")]
        if let Some(recorder) = std::mem::replace(&mut self.recorder, None) {
            recorder.stop().unwrap();
        }
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
    return mac::has_permission();

    #[cfg(not(target_os = "macos"))]
    true
}
