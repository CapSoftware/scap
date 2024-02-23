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

pub struct RGB8Frame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
}

pub struct RGBxFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

pub struct XBGRFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

pub struct BGRxFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}
pub struct BGRFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

pub struct BGRAFrame {
    pub display_time: u64,
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FrameType {
    #[default]
    YUVFrame,
    BGR0,
    RGB, // Prefer BGR0 because RGB is slower
    BGRAFrame,
}

pub enum Frame {
    YUVFrame(YUVFrame),
    RGB(RGBFrame),
    RGBx(RGBxFrame),
    XBGR(XBGRFrame),
    BGRx(BGRxFrame),
    BGR0(BGRFrame),
    BGRA(BGRAFrame),
}

pub enum FrameData<'a> {
    NV12(&'a YUVFrame),
    BGR0(&'a [u8]),
}

pub fn remove_alpha_channel(frame_data: Vec<u8>) -> Vec<u8> {
    let width = frame_data.len();
    let width_without_alpha = (width / 4) * 3;

    let mut data: Vec<u8> = vec![0; width_without_alpha];

    for (src, dst) in frame_data.chunks_exact(4).zip(data.chunks_exact_mut(3)) {
        dst[0] = src[0];
        dst[1] = src[1];
        dst[2] = src[2];
    }

    return data;
}

pub fn convert_bgra_to_rgb(frame_data: Vec<u8>) -> Vec<u8> {
    let width = frame_data.len();
    let width_without_alpha = (width / 4) * 3;

    let mut data: Vec<u8> = vec![0; width_without_alpha];

    for (src, dst) in frame_data.chunks_exact(4).zip(data.chunks_exact_mut(3)) {
        dst[0] = src[2];
        dst[1] = src[1];
        dst[2] = src[0];
    }

    return data;
}

pub fn get_cropped_data(data: Vec<u8>, cur_width: i32, height: i32, width: i32) -> Vec<u8> {
    if data.len() as i32 != height * cur_width * 4 {
        return data;
    } else {
        let mut cropped_data: Vec<u8> = vec![0; (4 * height * width).try_into().unwrap()];
        let mut cropped_data_index = 0;

        for i in 0..data.len() {
            let x = i as i32 % (cur_width * 4);
            if x < (width * 4) {
                cropped_data[cropped_data_index] = data[i];
                cropped_data_index += 1;
            }
        }
        return cropped_data;
    }
}
