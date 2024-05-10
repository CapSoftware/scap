mod engine;

use std::sync::mpsc;

use crate::{
    device::display,
    frame::{Frame, FrameType},
};

#[derive(Debug, Clone, Copy, Default)]
pub enum Resolution {
    _480p,
    _720p,
    _1080p,
    _1440p,
    _2160p,
    _4320p,

    #[default]
    Captured,
}

impl Resolution {
    fn value(&self, aspect_ratio: f32) -> [u32; 2] {
        match *self {
            Resolution::_480p => [640, ((640 as f32) / aspect_ratio).floor() as u32],
            Resolution::_720p => [1280, ((1280 as f32) / aspect_ratio).floor() as u32],
            Resolution::_1080p => [1920, ((1920 as f32) / aspect_ratio).floor() as u32],
            Resolution::_1440p => [2560, ((2560 as f32) / aspect_ratio).floor() as u32],
            Resolution::_2160p => [3840, ((3840 as f32) / aspect_ratio).floor() as u32],
            Resolution::_4320p => [7680, ((7680 as f32) / aspect_ratio).floor() as u32],
            Resolution::Captured => {
                panic!(".value should not be called when Resolution type is Captured")
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Clone)]
pub struct CGSize {
    pub width: f64,
    pub height: f64,
}
#[derive(Debug, Default, Clone)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}

/// Options passed to the screen capturer
#[derive(Debug, Default, Clone)]
pub struct Options {
    pub fps: u32,
    pub show_cursor: bool,
    pub show_highlight: bool,
    pub targets: Vec<display::Target>,

    // excluded targets will only work on macOS
    pub excluded_targets: Option<Vec<display::Target>>,
    // excluded windows will only work on macOS
    pub excluded_windows: Option<Vec<String>>,
    pub output_type: FrameType,
    pub output_resolution: Resolution,
    pub source_rect: Option<CGRect>,
}

/// Screen capturer class
pub struct Capturer {
    engine: engine::Engine,
    rx: mpsc::Receiver<Frame>,
}

impl Capturer {
    /// Create a new capturer instance with the provided options
    pub fn new(options: Options) -> Capturer {
        let (tx, rx) = mpsc::channel::<Frame>();
        let engine = engine::Engine::new(&options, tx);

        Capturer { engine, rx }
    }

    // TODO
    // Prevent starting capture if already started
    /// Start capturing the frames
    pub fn start_capture(&mut self) {
        self.engine.start();
    }

    /// Stop the capturer
    pub fn stop_capture(&mut self) {
        self.engine.stop();
    }

    /// Get the next captured frame
    pub fn get_next_frame(&self) -> Result<Frame, mpsc::RecvError> {
        self.rx.recv()
    }

    /// Get the dimensions the frames will be captured in
    pub fn get_output_frame_size(&mut self) -> [u32; 2] {
        self.engine.get_output_frame_size()
    }
}
