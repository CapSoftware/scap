use std::sync::mpsc;

use crate::frame::Frame;
use super::Options;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

pub struct Engine {
    #[cfg(target_os = "macos")]
    mac: screencapturekit::sc_stream::SCStream,

    #[cfg(target_os = "windows")]
    win: win::WinStream,
}

impl Engine {
    pub fn new(options: &Options, tx: mpsc::Sender<Frame>) -> Engine {
        #[cfg(target_os = "macos")]
        {
            let mac = mac::create_capturer(&options, tx);
            return Engine { mac }
        }

        #[cfg(target_os = "windows")]
        {
            let win = win::create_capturer(&options, tx);
            return Engine { win }
        }
    }

    pub fn start(&self) {
        #[cfg(target_os = "macos")]
        {
            // self.mac.add_output(Capturer::new(tx));
            self.mac.start_capture();
        }

        #[cfg(target_os = "windows")]
        {
            self.win.start_capture();
        }
    }

    pub fn stop(&self) {
        #[cfg(target_os = "macos")]
        {
            self.mac.stop_capture();
        }
    }
}