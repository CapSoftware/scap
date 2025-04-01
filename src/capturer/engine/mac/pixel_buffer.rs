use core_foundation::base::TCFType;
use core_media::time::CMTime;
// use core_media_rs::cm_time::CMTime;
use screencapturekit::{
    output::{CMSampleBuffer, CMSampleBufferRef},
    stream::output_type::SCStreamOutputType,
};
use std::sync::mpsc;

use crate::capturer::RawCapturer;

impl RawCapturer<'_> {
    pub fn get_next_sample_buffer(
        &self,
    ) -> Result<(CMSampleBuffer, SCStreamOutputType), mpsc::RecvError> {
        use std::time::Duration;

        let capturer = &self.capturer;

        loop {
            let error_flag = capturer
                .engine
                .error_flag
                .load(std::sync::atomic::Ordering::Relaxed);
            if error_flag {
                return Err(mpsc::RecvError);
            }

            return match capturer.rx.recv_timeout(Duration::from_millis(10)) {
                Ok(v) => Ok(v),
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => Err(mpsc::RecvError),
            };
        }
    }
}

pub fn get_sample_buffer_pts(sample_buffer: &CMSampleBuffer) -> CMTime {
    extern "C" {
        pub fn CMSampleBufferGetPresentationTimeStamp(sample: CMSampleBufferRef) -> CMTime;

    }

    unsafe { CMSampleBufferGetPresentationTimeStamp(sample_buffer.as_concrete_TypeRef()) }
}
