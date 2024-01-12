pub struct YUVFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub luminance_bytes: Vec<u8>,
    pub luminance_stride: i32,
    pub chrominance_bytes: Vec<u8>,
    pub chrominance_stride: i32,
}

pub enum Frame {
    YUVFrame(YUVFrame)
}

pub enum FrameData<'a> {
    NV12(&'a YUVFrame),
    BGR0(&'a [u8]),
}

