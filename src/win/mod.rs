#[cfg(target_os = "windows")]
use windows::Graphics::Capture;
use windows::Win32::Graphics::Direct3D11;

pub async fn main() {
    let supported = Capture::GraphicsCaptureSession::IsSupported().unwrap();

    if supported == false {
        // Show message to user that screen capture is unsupported
        return;
    }

    println!("Supported: {}", supported);

    //     // Create the D3D device and SharpDX device
    //     unsafe {
    //         let _device = Direct3D11::D3D11CreateDevice();
    //     }

    // Let the user pick an item to capture
    let picker = Capture::GraphicsCapturePicker::new().expect("Failed to create picker");

    let capture_item = picker.PickSingleItemAsync();

    capture_item.await;

    println!("Capture Item: {:?}", capture_item);
}
