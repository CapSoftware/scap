use std::{alloc::System, time::SystemTime};

pub struct AudioFrame {
    format: AudioFormat,
    channels: u16,
    is_planar: bool,
    data: Vec<u8>,
    sample_count: usize,
    rate: u32,
    timestamp: SystemTime,
}

impl AudioFrame {
    pub(crate) fn new(
        format: AudioFormat,
        channels: u16,
        is_planar: bool,
        data: Vec<u8>,
        sample_count: usize,
        rate: u32,
        timestamp: SystemTime,
    ) -> Self {
        assert!(data.len() >= sample_count * format.sample_size() as usize * channels as usize);

        Self {
            format,
            channels,
            is_planar,
            data,
            sample_count,
            rate,
            timestamp,
        }
    }

    pub fn format(&self) -> AudioFormat {
        self.format
    }

    pub fn planes(&self) -> u16 {
        if self.is_planar {
            self.channels
        } else {
            1
        }
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn is_planar(&self) -> bool {
        self.is_planar
    }

    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn time(&self) -> SystemTime {
        self.timestamp
    }

    pub fn plane_data(&self, plane: usize) -> &[u8] {
        if !self.is_planar {
            return &self.data;
        } else {
            let plane_size = self.sample_count * self.format.sample_size() as usize;
            let base = plane * plane_size;
            &self.data[base..base + plane_size]
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl AudioFormat {
    pub fn sample_size(&self) -> usize {
        match self {
            Self::I8 => std::mem::size_of::<i8>(),
            Self::I16 => std::mem::size_of::<i16>(),
            Self::I32 => std::mem::size_of::<i32>(),
            Self::I64 => std::mem::size_of::<i64>(),
            Self::U8 => std::mem::size_of::<u8>(),
            Self::U16 => std::mem::size_of::<u16>(),
            Self::U32 => std::mem::size_of::<u32>(),
            Self::U64 => std::mem::size_of::<u64>(),
            Self::F32 => std::mem::size_of::<f32>(),
            Self::F64 => std::mem::size_of::<f64>(),
            // v => panic!("Sample format {v:?} not supported"),
        }
    }
}

#[cfg(windows)]
impl From<cpal::SampleFormat> for AudioFormat {
    fn from(value: cpal::SampleFormat) -> Self {
        match value {
            cpal::SampleFormat::F32 => Self::F32,
            cpal::SampleFormat::F64 => Self::F64,
            cpal::SampleFormat::I8 => Self::I8,
            cpal::SampleFormat::I16 => Self::I16,
            cpal::SampleFormat::I32 => Self::I32,
            cpal::SampleFormat::I64 => Self::I64,
            cpal::SampleFormat::U8 => Self::U8,
            cpal::SampleFormat::U16 => Self::U16,
            cpal::SampleFormat::U32 => Self::U32,
            cpal::SampleFormat::U64 => Self::U64,
            _ => panic!("sample format {value:?} not supported"),
        }
    }
}
