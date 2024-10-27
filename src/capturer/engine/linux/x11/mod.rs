use std::sync::mpsc::Sender;

use crate::{capturer::Options, frame::Frame};

use super::LinuxCapturerImpl;

pub struct X11Capturer {
}

impl X11Capturer {
    pub fn new(_options: &Options, _tx: Sender<Frame>) -> Self {
        Self {}
    }
}

impl LinuxCapturerImpl for X11Capturer {
    fn start_capture(&mut self) {
        todo!()
    }

    fn stop_capture(&mut self) {
        todo!()
    }
}
