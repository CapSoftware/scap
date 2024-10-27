
use std::{env, sync::mpsc};

use pw::PwCapturer;
use x11::X11Capturer;

use crate::{
    capturer::Options,
    frame::Frame,
};

mod error;

mod pw;
mod x11;

pub trait LinuxCapturerImpl {
    fn start_capture(&mut self);
    fn stop_capture(&mut self);
}

pub struct LinuxCapturer {
    pub imp: Box<dyn LinuxCapturerImpl>,
}

type Type = mpsc::Sender<Frame>;

impl LinuxCapturer {
    pub fn new(options: &Options, tx: Type) -> Self {
        if env::var("WAYLAND_DISPLAY").is_ok() {
            println!("[DEBUG] On wayland");
            return Self {
                imp: Box::new(PwCapturer::new(options, tx)),
            };
        } else if env::var("DISPLAY").is_ok() {
            println!("[DEBUG] On X11");
            return Self {
                imp: Box::new(X11Capturer::new(options, tx)),
            };
        } else {
            panic!("Unsupported platform. Could not detect Wayland or X11 displays")
        }
    }
}

pub fn create_capturer(options: &Options, tx: mpsc::Sender<Frame>) -> LinuxCapturer {
    LinuxCapturer::new(options, tx)
}
