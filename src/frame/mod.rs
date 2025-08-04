mod audio;
mod video;

pub use audio::*;
pub use video::*;

pub enum Frame {
    Audio(AudioFrame),
    Video(VideoFrame),
}
