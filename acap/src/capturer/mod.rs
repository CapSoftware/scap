use std::sync::mpsc;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Stream, SupportedStreamConfig,
};

use crate::frame::Frame;

pub struct InputOptions {
    pub target: Device,
}

pub struct CapturerOptions {
    pub target: Device,
    pub config: SupportedStreamConfig,
}
pub struct Capturer {
    options: CapturerOptions,
    pub stream: Option<Stream>,
    tx: mpsc::Sender<Frame<f32>>,
    rx: mpsc::Receiver<Frame<f32>>,
}

impl Capturer {
    pub fn new() -> Capturer {
        let (tx, rx) = mpsc::channel::<Frame<f32>>();

        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        let config = device.default_input_config().unwrap();
        Capturer {
            options: CapturerOptions {
                target: device,
                config,
            },
            stream: None,
            tx,
            rx,
        }
    }

    pub fn start_capture(&mut self) {
        let device = &self.options.target;
        let config = self.options.config.clone();
        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };
        let tx2 = self.tx.clone();
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| {
                        let data = data.to_vec();

                        let _ = &tx2.send(Frame { data });
                    },
                    err_fn,
                    None,
                )
                .unwrap(),
            _ => panic!("Unknown stream format"),
        };
        let _ = stream.play();
        self.stream = Some(stream);
    }

    pub fn pause_capture(&self) {}

    pub fn stop_capture(self) {
        let stream = self.stream.unwrap();
        drop(stream);
    }

    pub fn get_next_frame(&self) -> Result<Frame<f32>, mpsc::RecvError> {
        self.rx.recv()
    }
}
