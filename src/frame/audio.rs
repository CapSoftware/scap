pub struct AudioFrame {
    format: AudioFormat,
    channels: u16,
    is_planar: bool,
    data: Vec<u8>,
    sample_count: usize,
    rate: u32,
}

impl AudioFrame {
    pub(crate) fn new(
        format: AudioFormat,
        channels: u16,
        is_planar: bool,
        data: Vec<u8>,
        sample_count: usize,
        rate: u32,
    ) -> Self {
        assert_eq!(
            data.len(),
            sample_count * format.sample_size() as usize * channels as usize
        );

        Self {
            format,
            channels,
            is_planar,
            data,
            sample_count,
            rate,
        }
    }

    pub fn planes(&self) -> u16 {
        if self.is_planar {
            self.channels
        } else {
            1
        }
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
            v => panic!("Sample format {v:?} not supported"),
        }
    }
}
