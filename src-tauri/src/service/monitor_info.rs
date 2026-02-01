use serde::Serialize;
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

// Constant if not found in Gdi
const MONITORINFOF_PRIMARY: u32 = 0x00000001;

#[derive(Debug, Clone, Serialize)]
pub struct MonitorData {
    pub id: usize,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub dpi_scale: f32,
    pub is_primary: bool,
}

impl MonitorData {
    pub fn get_logical_rect(&self) -> (f64, f64, f64, f64) {
        let s = self.dpi_scale as f64;
        (
            self.x as f64 / s,
            self.y as f64 / s,
            self.width as f64 / s,
            self.height as f64 / s,
        )
    }
}

struct EnumState {
    monitors: Vec<MonitorData>,
}

unsafe extern "system" fn enum_monitor_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let state = &mut *(lparam.0 as *mut EnumState);

    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut info.monitorInfo as *mut _ as *mut _).as_bool() {
        let mut dpi_x = 0;
        let mut dpi_y = 0;
        let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);

        let dpi_scale = dpi_x as f32 / 96.0;

        let rect = info.monitorInfo.rcMonitor;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        let name = String::from_utf16_lossy(&info.szDevice)
            .trim_matches(char::from(0))
            .to_string();

        let is_primary = (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;

        state.monitors.push(MonitorData {
            id: state.monitors.len(),
            name,
            x: rect.left,
            y: rect.top,
            width,
            height,
            dpi_scale,
            is_primary,
        });
    }

    BOOL(1)
}

pub fn enumerate_monitors() -> Vec<MonitorData> {
    let mut state = EnumState {
        monitors: Vec::new(),
    };

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitor_proc),
            LPARAM(&mut state as *mut _ as isize),
        );
    }

    state.monitors
}

pub fn is_dpi_uniform(monitors: &[MonitorData]) -> bool {
    if monitors.is_empty() {
        return true;
    }
    let first_dpi = monitors[0].dpi_scale;
    monitors
        .iter()
        .all(|m| (m.dpi_scale - first_dpi).abs() < 0.01)
}
