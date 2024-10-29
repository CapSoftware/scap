use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        mpsc::Sender,
        Arc,
    }, thread::JoinHandle
};

use xcb::x;

use crate::{capturer::Options, frame::Frame, targets::linux::get_default_x_display, Target};

use super::{error::LinCapError, LinuxCapturerImpl};

pub struct X11Capturer {
    capturer_join_handle: Option<JoinHandle<Result<(), LinCapError>>>,
    capturer_state: Arc<AtomicU8>,
}

impl X11Capturer {
    pub fn new(options: &Options, tx: Sender<Frame>) -> Self {
        let (conn, screen_num) =
            xcb::Connection::connect_with_extensions(None, &[xcb::Extension::RandR], &[]).unwrap();
        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num as usize).unwrap();

        let target = match &options.target {
            Some(t) => t.clone(),
            None => Target::Display(get_default_x_display(&conn, screen).unwrap()),
        };

        let framerate = options.fps as f32;
        let capturer_state = Arc::new(AtomicU8::new(0));
        let capturer_state_clone = Arc::clone(&capturer_state);

        let jh = std::thread::spawn(move || {
            while capturer_state_clone.load(Ordering::Acquire) == 0 {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            let frame_time = std::time::Duration::from_secs_f32(1.0 / framerate);
            while capturer_state_clone.load(Ordering::Acquire) == 1 {
                let start = std::time::Instant::now();
                match &target {
                    Target::Window(win) => {
                        let geom_cookie = conn.send_request(&x::GetGeometry {
                            drawable: x::Drawable::Window(win.raw_handle),
                        });
                        let geom = conn.wait_for_reply(geom_cookie).unwrap();

                        let img_cookie = conn.send_request(&x::GetImage {
                            format: x::ImageFormat::ZPixmap,
                            drawable: x::Drawable::Window(win.raw_handle),
                            x: 0,
                            y: 0,
                            width: geom.width(),
                            height: geom.height(),
                            plane_mask: u32::MAX,
                        });
                        let img = conn.wait_for_reply(img_cookie).unwrap();

                        let img_data = img.data();

                        tx.send(Frame::BGRx(crate::frame::BGRxFrame {
                            display_time: 0,
                            width: geom.width() as i32,
                            height: geom.height() as i32,
                            data: img_data.to_vec(),
                        })).unwrap();
                    }
                    Target::Display(disp) => {
                        let img_cookie = conn.send_request(&x::GetImage {
                            format: x::ImageFormat::ZPixmap,
                            drawable: x::Drawable::Window(disp.raw_handle),
                            x: disp.x_offset,
                            y: disp.y_offset,
                            width: disp.width,
                            height: disp.height,
                            plane_mask: u32::MAX,
                        });
                        let img = conn.wait_for_reply(img_cookie).unwrap();

                        let img_data = img.data();

                        tx.send(Frame::BGRx(crate::frame::BGRxFrame {
                            display_time: 0,
                            width: disp.width as i32,
                            height: disp.height as i32,
                            data: img_data.to_vec(),
                        })).unwrap();
                    }
                }
                let elapsed = start.elapsed();
                if elapsed < frame_time {
                    std::thread::sleep(frame_time - start.elapsed());
                }
            }

            Ok(())
        });

        Self {
            capturer_state: capturer_state,
            capturer_join_handle: Some(jh),
        }
    }
}

impl LinuxCapturerImpl for X11Capturer {
    fn start_capture(&mut self) {
        self.capturer_state.store(1, Ordering::Release);
    }

    fn stop_capture(&mut self) {
        self.capturer_state.store(2, Ordering::Release);
        if let Some(handle) = self.capturer_join_handle.take() {
            if let Err(e) = handle.join().expect("Failed to join capturer thread") {
                eprintln!("Error occured capturing: {e}");
            }
        }
    }
}
