use windows::core::BOOL;
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub rect: RECT,
    pub dpi_x: u32,
    pub dpi_y: u32,
    pub is_primary: bool,
}

struct MonitorEnumContext {
    monitors: Vec<MonitorInfo>,
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let context = &mut *(lparam.0 as *mut MonitorEnumContext);

    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut info.monitorInfo as *mut _ as *mut _).as_bool() {
        let mut dpi_x = 0;
        let mut dpi_y = 0;
        let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);

        let name = String::from_utf16_lossy(&info.szDevice)
            .trim_matches('\0')
            .to_string();

        // MONITORINFOF_PRIMARY is 1
        let is_primary = (info.monitorInfo.dwFlags & 1) != 0;

        context.monitors.push(MonitorInfo {
            name,
            rect: info.monitorInfo.rcMonitor,
            dpi_x,
            dpi_y,
            is_primary,
        });
    }

    BOOL(1)
}

pub fn enumerate_monitors() -> anyhow::Result<Vec<MonitorInfo>> {
    let mut context = MonitorEnumContext {
        monitors: Vec::new(),
    };

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut context as *mut _ as isize),
        );
    }

    Ok(context.monitors)
}
