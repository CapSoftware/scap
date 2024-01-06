mod encoder;
mod audio;
mod utils;
mod output;
mod frame;
mod device;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod win;

use std::{time::Duration, fs::File};

use ac_ffmpeg::{format::muxer::Muxer, time::{Timestamp, TimeBase}};
use encoder::Encoder;
use crate::{
    output::open_output,
    frame::{
        YUVFrame,
        FrameData
    },
    encoder::config::{libx264, InputConfig},
    device::display, mac::Capturer
};

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

#[derive(Debug)]
pub struct Options {
    pub fps: u32,
    pub show_cursor: bool,
    pub show_highlight: bool,
    pub targets: Vec<Target>,

    // excluded targets will only work on macOS
    pub excluded_targets: Option<Vec<Target>>,

    // TODO: move this somewhere more appropriate
    pub output_filename: String,
}

pub struct Recorder {
    encoder: Encoder,
    file_output_muxer: Muxer<File>,
    audio_recorder: audio::AudioRecorder,
    options: Options,

    // private receiver here, transmitter
    #[cfg(target_os = "macos")]
    recorder: screencapturekit::sc_stream::SCStream,

    #[cfg(target_os = "windows")]
    recorder: Option<windows_capture::capture::CaptureControl>,
}

impl Recorder {
    pub fn init(options: Options) -> Self {
        let display = display::get_main_display();
        let display_id = display.display_id;

        let scale = display::get_scale_factor(display_id) as u32;
        let width = display.width * scale;
        let height = display.height * scale;

        let config = libx264();
        let encoder = Encoder::new(
            &InputConfig{ width: width as usize, height: height as usize },
            &config
        );

        let codec_parameters = encoder.codec_parameters();
        let file_output_muxer = open_output(&options.output_filename, &[codec_parameters]).unwrap();

        let audio_recorder = audio::AudioRecorder::new();


        #[cfg(target_os = "macos")]
        let recorder = mac::create_recorder(&options);

        #[cfg(target_os = "windows")]
        let recorder = None;

        Recorder {
            encoder,
            file_output_muxer,
            audio_recorder,
            recorder,
            options,
        }
    }

    pub fn start_capture(&mut self) {
        // start receiver here
        let (tx, rx) = std::sync::mpsc::channel::<YUVFrame>();

        self.audio_recorder.start_recording();

        #[cfg(target_os = "macos")]
        {
            self.recorder.add_output(Capturer::new(tx));
            self.recorder.start_capture();
        }

        #[cfg(target_os = "windows")]
        {
            let recorder = win::create_recorder(&options, tx);
            self.recorder = Some(recorder);
        }

        let duration = Duration::from_secs_f32(20.0);
        let time_base = TimeBase::new(1, 25);
        let mut frame_idx = 0;
        let mut frame_timestamp = Timestamp::new(frame_idx, time_base);
        let max_timestamp = Timestamp::from_millis(0) + duration;
        while frame_timestamp < max_timestamp {
            let frame = rx.recv().unwrap();
            self.encoder
                    .encode_and_save_to_file(FrameData::NV12(&frame), frame_timestamp, &mut self.file_output_muxer)
                    .unwrap();
            frame_idx += 1;
            frame_timestamp = Timestamp::new(frame_idx, time_base);
        }
    }

    pub fn stop_capture(&mut self) {
        self.audio_recorder.stop_recording();

        #[cfg(target_os = "macos")]
        self.recorder.stop_capture();

        // TODO: temporary workaround until I find a better way
        #[cfg(target_os = "windows")]
        if let Some(recorder) = std::mem::replace(&mut self.recorder, None) {
            recorder.stop().unwrap();
        }

        self.encoder.flush().unwrap();
        while let Some(packet) = self.encoder.take().unwrap() {
            self.file_output_muxer.push(packet.with_stream_index(0)).unwrap();
        }

        self.file_output_muxer.flush().unwrap();
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
