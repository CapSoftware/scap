mod engine;

use std::sync::mpsc;

use crate::{device::display, frame::Frame};

#[derive(Debug)]
pub struct Options {
    pub fps: u32,
    pub show_cursor: bool,
    pub show_highlight: bool,
    pub targets: Vec<display::Target>,

    // excluded targets will only work on macOS
    pub excluded_targets: Option<Vec<display::Target>>,
}

pub struct Capturer {
    options: Options,
    engine: engine::Engine,
    rx: mpsc::Receiver<Frame>,
}

impl Capturer {
    pub fn new(options: Options) -> Capturer {
        let (tx, rx) = mpsc::channel::<Frame>();
        let engine = engine::Engine::new(&options, tx);

        Capturer { options, engine, rx }
    }

    // TODO
    // Prevent starting capture if already started
    // Tx,Rx should be of type frame
    pub fn start_capture(&self) {

        self.engine.start();
    }

    pub fn stop_capture(&self) {
        self.engine.stop();
    }

    pub fn get_next_frame(&self) -> Result<Frame, mpsc::RecvError> {
        self.rx.recv()
    }
}