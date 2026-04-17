use windows::Win32::Foundation::{HWND, POINT, SIZE, COLORREF};
use windows::Win32::Graphics::Gdi::{AC_SRC_OVER, BLENDFUNCTION};
use windows::Win32::UI::WindowsAndMessaging::{WS_EX_LAYERED, ULW_ALPHA, UpdateLayeredWindow, GetWindowLongW, SetWindowLongW, GWL_EXSTYLE};
use crate::service::win32::gdi::SafeHDC;
use super::types::SafeHWND;

pub fn update_layered_window(
    hwnd: &SafeHWND,
    hdc_src: &SafeHDC,
    point: &POINT,
    size: &SIZE,
    alpha: u8,
    alpha_format: u8,
) -> anyhow::Result<()> {
    unsafe {
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: alpha,
            AlphaFormat: alpha_format,
        };

        UpdateLayeredWindow(
            hwnd.0,
            None,
            Some(point),
            Some(size),
            Some(hdc_src.0),
            Some(&POINT { x: 0, y: 0 }),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        )?;
        Ok(())
    }
}

pub fn enable_transparency_composition(hwnd: &SafeHWND) -> anyhow::Result<()> {
    unsafe {
        use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
        use windows::Win32::UI::Controls::MARGINS;
        let margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        DwmExtendFrameIntoClientArea(hwnd.0, &margins)?;
        Ok(())
    }
}

pub fn disable_transparency_composition(hwnd: &SafeHWND) -> anyhow::Result<()> {
    unsafe {
        use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
        use windows::Win32::UI::Controls::MARGINS;
        let margins = MARGINS {
            cxLeftWidth: 0,
            cxRightWidth: 0,
            cyTopHeight: 0,
            cyBottomHeight: 0,
        };
        DwmExtendFrameIntoClientArea(hwnd.0, &margins)?;
        Ok(())
    }
}

pub fn set_layered_attribute(hwnd: &SafeHWND, layered: bool) -> anyhow::Result<()> {
    unsafe {
        let style = GetWindowLongW(hwnd.0, GWL_EXSTYLE);
        let layered_flag = WS_EX_LAYERED.0 as i32;

        let new_style = if layered {
            style | layered_flag
        } else {
            style & !layered_flag
        };

        if style != new_style {
            SetWindowLongW(hwnd.0, GWL_EXSTYLE, new_style);
        }
        Ok(())
    }
}

pub fn apply_theme(hwnd: HWND, is_dark: bool) -> anyhow::Result<()> {
    unsafe {
        use windows::Win32::Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_CAPTION_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE,
        };

        // 1. Set Immersive Dark Mode
        let value: i32 = if is_dark { 1 } else { 0 };
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &value as *const i32 as *const _,
            4,
        )?;

        // 2. Set Caption Color
        let color: u32 = if is_dark { 0x000A0A0A } else { 0x00FFFFFF };
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &color as *const u32 as *const _,
            4,
        )?;

        Ok(())
    }
}
