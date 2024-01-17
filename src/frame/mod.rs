pub struct YUVFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub luminance_bytes: Vec<u8>,
    pub luminance_stride: i32,
    pub chrominance_bytes: Vec<u8>,
    pub chrominance_stride: i32,
}

pub struct RGBFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

pub struct BGRFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FrameType {
    #[default]
    YUVFrame,
    BGR0,
    RGB, // Prefer BGR0 because RGB is slower
}

pub enum Frame {
    YUVFrame(YUVFrame),
    BGR0(BGRFrame),
    RGB(RGBFrame),
}

pub enum FrameData<'a> {
    NV12(&'a YUVFrame),
    BGR0(&'a [u8]),
}

