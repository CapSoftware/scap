use windows::Graphics::Capture;
use windows::Win32::Graphics::Direct3D11;

pub fn main() {
    //     // Create the D3D device and SharpDX device
    //     unsafe {
    //         let _device = Direct3D11::D3D11CreateDevice();
    //     }

    // Let the user pick an item to capture
    let picker = Capture::GraphicsCapturePicker::new().expect("Failed to create picker");

    let capture_item = picker.PickSingleItemAsync();

    // capture_item.await;

    println!("Capture Item: {:?}", capture_item);
}

pub fn is_supported() -> bool {
    Capture::GraphicsCaptureSession::IsSupported().unwrap()
}
