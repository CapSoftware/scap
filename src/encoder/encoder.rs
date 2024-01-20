use std::fs::File;

use ac_ffmpeg::codec::video::VideoEncoder;
use ac_ffmpeg::codec::{video, CodecParameters, Encoder as FFmpegEncoder};
use ac_ffmpeg::format::muxer::Muxer;
use ac_ffmpeg::packet::Packet;
use ac_ffmpeg::time::{TimeBase, Timestamp};

use crate::encoder::utils;
use crate::utils::Result;
use crate::FrameData;

use super::config::{EncoderConfig, InputConfig};
use super::frame_pool::FramePool;

pub struct Encoder {
    encoder: VideoEncoder,
    encoder_config: EncoderConfig,
    frame_pool: FramePool,
    input_config: InputConfig,
}

impl Encoder {
    pub fn new(input_config: &InputConfig, encoder_config: &EncoderConfig) -> Self {
        let width = if input_config.width % 2 == 0 {
            input_config.width
        } else {
            input_config.width + 1
        };
        let height = if input_config.height % 2 == 0 {
            input_config.height
        } else {
            input_config.height + 1
        };

        let time_base = TimeBase::new(1, 90_000); // Maybe move to config?

        let pixel_format = video::frame::get_pixel_format(&encoder_config.pixel_format);

        let mut encoder = VideoEncoder::builder(&encoder_config.encoder)
            .unwrap()
            .pixel_format(pixel_format)
            .width(width)
            .height(height)
            .time_base(time_base);

        for option in &encoder_config.options {
            encoder = encoder.set_option(option.0, option.1);
        }

        let encoder = encoder.build().unwrap();

        Self {
            encoder,
            encoder_config: encoder_config.clone(),
            frame_pool: FramePool::new(width, height, time_base, pixel_format),
            input_config: input_config.clone(),
        }
    }

    pub fn encode_and_save_to_file(
        &mut self,
        frame_data: FrameData,
        frame_time: Timestamp,
        file_muxer: &mut Muxer<File>,
    ) -> Result<()> {
        let mut frame = self.frame_pool.take();
        frame = frame.with_pts(frame_time).with_picture_type(
            if self
                .encoder_config
                .force_idr
                .swap(false, std::sync::atomic::Ordering::Relaxed)
            {
                video::frame::PictureType::I
            } else {
                video::frame::PictureType::None
            },
        );

        match frame_data {
            FrameData::NV12(nv12) => {
                assert_eq!(self.encoder_config.pixel_format, "nv12");
                let encoder_buffer_len = frame.planes_mut()[0].data_mut().len();
                let encoder_line_size = encoder_buffer_len / self.input_config.height;
                let encoder_num_lines = self.input_config.width;

                utils::copy_nv12(
                    &nv12.luminance_bytes,
                    nv12.luminance_stride as usize,
                    encoder_line_size,
                    encoder_num_lines,
                    frame.planes_mut()[0].data_mut(),
                );
                utils::copy_nv12(
                    &nv12.chrominance_bytes,
                    nv12.chrominance_stride as usize,
                    encoder_line_size,
                    encoder_num_lines,
                    frame.planes_mut()[1].data_mut(),
                );
            }
            FrameData::BGR0(bgr0) => match self.encoder_config.pixel_format.as_str() {
                "bgra" => {
                    frame.planes_mut()[0].data_mut().copy_from_slice(bgr0);
                }
                _ => unimplemented!(),
            },
        }
        let frame = frame.freeze();

        self.encoder.push(frame.clone())?;
        self.frame_pool.put(frame);
        while let Some(packet) = self.encoder.take()? {
            file_muxer.push(packet.with_stream_index(0))?;
        }
        Ok(())
    }

    pub fn codec_parameters(&self) -> CodecParameters {
        return self.encoder.codec_parameters().into();
    }

    pub fn flush(&mut self) -> Result<()> {
        return Ok(self.encoder.flush()?);
    }

    pub fn take(&mut self) -> Result<Option<Packet>> {
        return Ok(self.encoder.take()?);
    }
}
