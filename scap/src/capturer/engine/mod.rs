use std::sync::mpsc;

use super::Options;
use crate::frame::Frame;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "linux")]
mod linux;

pub struct Engine {
    #[cfg(target_os = "macos")]
    mac: screencapturekit::sc_stream::SCStream,

    #[cfg(target_os = "windows")]
    win: win::WinStream,

    #[cfg(target_os = "linux")]
    linux: linux::LinuxCapturer,
}

impl Engine {
    pub fn new(options: &Options, tx: mpsc::Sender<Frame>) -> Engine {
        #[cfg(target_os = "macos")]
        {
            let mac = mac::create_capturer(&options, tx);
            return Engine { mac };
        }

        #[cfg(target_os = "windows")]
        {
            let win = win::create_capturer(&options, tx);
            return Engine { win };
        }

        #[cfg(target_os = "linux")]
        {
            let linux = linux::create_capturer(&options, tx);
            return Engine { linux };
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

        #[cfg(target_os = "linux")]
        {
            self.linux.start_capture();
        }
    }

    pub fn stop(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.mac.stop_capture();
        }

        #[cfg(target_os = "linux")]
        {
            self.linux.stop_capture();
        }
    }
}
