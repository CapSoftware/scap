use super::Target;

use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFOEXW};
use windows_capture::{
    graphics_capture_api::GraphicsCaptureApi,
    monitor::Monitor,
    window::Window,
};

fn get_monitor_name(h_monitor: HMONITOR) -> windows::core::Result<String> {
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    let success =
        unsafe { GetMonitorInfoW(h_monitor, &mut monitor_info as *mut _ as *mut _).as_bool() };

    if success {
        let len = monitor_info
            .szDevice
            .iter()
            .position(|&i| i == 0)
            .unwrap_or(0);
        let name = String::from_utf16(&monitor_info.szDevice[..len]).unwrap();

        let clean_name = match name.rfind('\\') {
            Some(index) => name.chars().skip(index + 1).collect(),
            None => name.to_string(),
        };

        Ok(clean_name)
    } else {
        Err(windows::core::Error::new(
            windows::core::HRESULT(0),
            "Failed to get monitor info".into(),
        ))
    }
}

pub fn is_supported() -> bool {
    GraphicsCaptureApi::is_supported().expect("Failed to check support")
}

// TODO: add correct permission mechanism here
pub fn has_permission() -> bool {
    true
}

pub fn get_targets() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    let displays = Monitor::enumerate().expect("Failed to enumerate monitors");

    let mut cnt = 1;
    for display in displays {
        let id = cnt;
        cnt = cnt + 1;
        let title = get_monitor_name(display.as_raw_hmonitor()).unwrap();

        let target = Target {
            id,
            title,
        };
        targets.push(target);
    }

    let windows = Window::enumerate().expect("Failed to enumerate windows");
    for window in windows {
        let handle = window.as_raw_hwnd();

        let title = window
            .title()
            .unwrap()
            .to_string();

        let target = Target {
            id: 3,
            title,
        };
        targets.push(target);
    }

    targets
}