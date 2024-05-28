use super::{Display, Target};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::{Foundation::HWND, Graphics::Gdi::HMONITOR};
use windows_capture::{monitor::Monitor, window::Window};

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

const BASE_DPI: u32 = 96;

pub fn get_scale_factor(display_id: u32) -> f64 {
    let hmonitor = HMONITOR(display_id as isize);
    let mut dpi_x = 0;
    let mut dpi_y = 0;

    let dpi = unsafe {
        if GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y).is_ok() {
            dpi_x.into()
        } else {
            BASE_DPI
        }
    };

    let scale_factor = dpi / BASE_DPI;
    scale_factor as f64
}
