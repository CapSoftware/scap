use windows::win32::windows_and_messaging::{GetDesktopWindow, CreateDXGIFactory2};
use windows::win32::direct3d11::{D3D11CreateDevice, D3D11CreateDeviceAndSwapChain, D3D_DRIVER_TYPE_HARDWARE, D3D11_SDK_VERSION, D3D11_CREATE_DEVICE_DEBUG};
use windows::win32::dxgi::{IDXGIFactory, DXGI_CREATE_FACTORY_DEBUG};
use windows::win32::com::CreateUri;
use windows::win32::storage::KnownFolders;
use windows::win32::system_services::HANDLE;
use windows::data::xml::DomDocument;
use windows::media::capture::{GraphicsCapturePicker, GraphicsCaptureSession, GraphicsCaptureItem};
use windows::media::capture::windows_graphics_capture::Direct3D11CaptureFramePool;
use windows::media::media_properties::{MediaVideoEncodingProperties, VideoEncodingQuality};
use windows::media::transcoding::{MediaTranscoder, TranscodeFailureReason};
use windows::foundation::{PropertyValue, TypedEventHandler};
use windows::foundation::numerics::{Vector2, Vector3, Vector4};
use windows::storage::{FileAccessMode, CreationCollisionOption};
use windows::storage::streams::{DataWriter, IRandomAccessStream};
use windows::media::core::{MediaEncodingProfile, MediaStreamSource, MediaStreamSourceStartingEventArgs, MediaStreamSourceSampleRequestedEventArgs, MediaStreamSourceStartingRequestedHandler, MediaStreamSourceSampleRequestedHandler};
use windows::foundation::TimeSpan;
use windows::foundation::TypedEventHandler_2;
use windows::media::effects::{VideoEffectDefinition, VideoTransformEffect};
use windows::media::capture::{MediaCaptureInitializationSettings, MediaCapture};
use windows::media::capture::mediacapture_device_information;
use std::collections::HashMap;
use windows::ui::core::{CoreDispatcher, CoreDispatcherPriority};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

#[cfg_attr(feature = "cargo-clippy", allow(clippy::unnecessary_wraps))]
pub fn create_d3d_device() -> Result<ComPtr<ID3D11Device>, windows::Error> {
    let mut d3d_device = ComPtr::null();
    let mut d3d_device_context = ComPtr::null();

    let hr = unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_DEBUG,
            &[D3D_FEATURE_LEVEL_11_0],
            1,
            D3D11_SDK_VERSION,
            &mut d3d_device,
            &mut D3D_FEATURE_LEVEL_11_0,
            &mut d3d_device_context,
        )
    };

    if hr < 0 {
        return Err(windows::Error::fast_error(hr));
    }

    Ok(d3d_device)
}

pub async fn setup_encoding() -> windows::Result<()> {
    if !GraphicsCaptureSession::is_supported()? {
        // Show message to user that screen capture is unsupported
        return Ok(());
    }

    let mut _device: Option<ComPtr<ID3D11Device>> = None;
    let mut _sharp_d3d_device: Option<ComPtr<Direct3D11CaptureFramePool>> = None;

    // Create the D3D device and SharpDX device
    if _device.is_none() {
        _device = Some(create_d3d_device()?);
    }

    // In Rust, we use Option instead of null checks

    // Let the user pick an item to capture
    let picker = GraphicsCapturePicker::new()?;
    let _capture_item = picker.pick_single_item_async()?.await?;

    // Initialize a blank texture and render target view for copying frames, using the same size as the capture item
    let width = _capture_item.size()?.width as u32;
    let height = _capture_item.size()?.height as u32;
    let (d3d_texture, d3d_render_target_view) = initialize_compose_texture(&_device.as_ref().unwrap(), width, height)?;

    // This example encodes video using the item's actual size.
    let width = _capture_item.size()?.width as u32;
    let height = _capture_item.size()?.height as u32;

    // Make sure the dimensions are even. Required by some encoders.
    let width = if width % 2 == 0 { width } else { width + 1 };
    let height = if height % 2 == 0 { height } else { height + 1 };

    let encoding_profile = MediaEncodingProfile::create_mp4(VideoEncodingQuality::HD1080p)?;
    let bitrate = encoding_profile.video()?.bitrate()?;
    let framerate = 30u32;

    let encoding_profile = {
        let mut profile = MediaEncodingProfile::new()?;
        profile.container()?.subtype("MPEG4")?;
        profile.video()?.subtype("H264")?;
        profile.video()?.width(width)?;
        profile.video()?.height(height)?;
        profile.video()?.bitrate(bitrate)?;
        profile.video()?.frame_rate()?.numerator(framerate)?;
        profile.video()?.frame_rate()?.denominator(1u32)?;
        profile.video()?.pixel_aspect_ratio()?.numerator(1)?;
        profile.video()?.pixel_aspect_ratio()?.denominator(1)?;
        profile
    };

    let video_properties = {
        let mut properties = MediaVideoEncodingProperties::create_uncompressed()?;
        properties.subtype(MediaEncodingSubtypes::Bgra8)?;
        properties.width(width)?;
        properties.height(height)?;
        properties
    };

    let video_descriptor = {
        let properties = video_properties.cast()?;
        let descriptor = MediaStreamDescriptor::video_desc(properties)?;
        descriptor
    };

    // Create our MediaStreamSource
    let media_stream_source = {
        let mut source = MediaStreamSource::new(video_descriptor)?;
        source.buffer_time(TimeSpan::from_seconds(0))?;
        source.starting(media_stream_source_starting_handler)?;
        source.sample_requested(media_stream_source_sample_requested_handler)?;
        source
    };

    // Create our transcoder
    let mut transcoder = MediaTranscoder::new()?;
    transcoder.hardware_acceleration_enabled(true)?;

    // Create a destination file - Access to the VideosLibrary requires the "Videos Library" capability
    let folder = KnownFolders::VideosLibrary()?;
    let name = format!("{:?}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
    let file = folder.create_file_async(format!("{}.mp4", name), CreationCollisionOption::GenerateUniqueName)?.await?;

    let stream = file.open_async(FileAccessMode::ReadWrite)?.await?;
    encode_async(stream)?;

    Ok(())
}

fn initialize_compose_texture(device: &ComPtr<ID3D11Device>, width: u32, height: u32) -> Result<(ComPtr<ID3D11Texture2D>, ComPtr<ID3D11RenderTargetView>), windows::Error> {
    let mut d3d_texture = ComPtr::null();
    let mut d3d_render_target_view = ComPtr::null();

    let texture_desc = D
