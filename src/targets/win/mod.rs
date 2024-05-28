use super::{Display, Target};
use windows_capture::{monitor::Monitor, window::Window};

pub use windows::Win32::{Foundation::HWND, Graphics::Gdi::HMONITOR};

use windows::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, LOGPIXELSX, LOGPIXELSY};

pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    // Add displays to targets
    let displays = Monitor::enumerate().expect("Failed to enumerate monitors");
    for display in displays {
        let id = display.as_raw_hmonitor() as u32;
        let title = display.device_name().expect("Failed to get monitor name");

        let target = Target::Display(super::Display {
            id,
            title,
            raw_handle: HMONITOR(display.as_raw_hmonitor()),
        });
        targets.push(target);
    }

    // Add windows to targets
    let windows = Window::enumerate().expect("Failed to enumerate windows");
    for window in windows {
        let id = window.as_raw_hwnd() as u32;
        let title = window.title().unwrap().to_string();

        let target = Target::Window(super::Window {
            id,
            title,
            raw_handle: HWND(window.as_raw_hwnd()),
        });
        targets.push(target);
    }

    targets
}

pub fn get_main_display() -> Display {
    let display = Monitor::primary().expect("Failed to get primary monitor");
    let id = display.as_raw_hmonitor() as u32;

    Display {
        id,
        title: display.device_name().expect("Failed to get monitor name"),
        raw_handle: HMONITOR(display.as_raw_hmonitor()),
    }
}

pub fn get_scale_factor() -> f64 {
    unsafe {
        let hdc = GetDC(None);

        let dpi_x = GetDeviceCaps(hdc, LOGPIXELSX);
        let dpi_y = GetDeviceCaps(hdc, LOGPIXELSY);

        ReleaseDC(None, hdc);

        let scale_x = dpi_x as f64 / 96.0;
        let scale_y = dpi_y as f64 / 96.0;
        let scale = (scale_x + scale_y) / 2.0;

        return scale;
    }
}
