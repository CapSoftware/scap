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
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

pub struct RGBxFrame {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FrameType {
    #[default]
    YUVFrame,
    BGR0,
    RGB,
}

pub enum Frame {
    YUVFrame(YUVFrame),
    BGR0(Vec<u8>),
    RGB(RGBFrame),
    RGBx(RGBxFrame),
}

pub enum FrameData<'a> {
    NV12(&'a YUVFrame),
    BGR0(&'a [u8]),
}

