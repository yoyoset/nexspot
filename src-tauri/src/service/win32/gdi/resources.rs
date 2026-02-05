use super::dc::SafeHDC;
use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Gdi::{
    AddFontResourceExW, CreateCompatibleBitmap, CreateFontW, CreateSolidBrush, DeleteObject,
    RemoveFontResourceExW, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, FR_PRIVATE,
    HBITMAP, HBRUSH, HFONT, HGDIOBJ, HPEN, OUT_DEFAULT_PRECIS, VARIABLE_PITCH,
};

#[derive(Debug)]
pub struct SafeHBITMAP(pub(crate) HBITMAP);

impl Drop for SafeHBITMAP {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(self.0 .0));
            }
        }
    }
}

#[derive(Debug)]
pub struct SafeBrush(pub(crate) HBRUSH);

impl Drop for SafeBrush {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(self.0 .0));
            }
        }
    }
}

#[derive(Debug)]
pub struct SafeFont(pub(crate) HFONT);

impl Drop for SafeFont {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(self.0 .0));
            }
        }
    }
}

#[derive(Debug)]
pub struct SafePen(pub(crate) HPEN);

impl Drop for SafePen {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(self.0 .0));
            }
        }
    }
}

pub fn create_compatible_bitmap(hdc: &SafeHDC, w: i32, h: i32) -> anyhow::Result<SafeHBITMAP> {
    unsafe {
        let h = CreateCompatibleBitmap(hdc.0, w, h);
        if h.is_invalid() {
            anyhow::bail!("Failed to create compatible bitmap");
        }
        Ok(SafeHBITMAP(h))
    }
}

pub fn create_solid_brush(color: u32) -> anyhow::Result<SafeBrush> {
    unsafe {
        let h = CreateSolidBrush(COLORREF(color));
        if h.is_invalid() {
            anyhow::bail!("Failed to create solid brush");
        }
        Ok(SafeBrush(h))
    }
}

pub fn create_font(height: i32, weight: i32, name: &str) -> anyhow::Result<SafeFont> {
    unsafe {
        let name_u16: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let h = CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            VARIABLE_PITCH.0 as u32,
            windows::core::PCWSTR(name_u16.as_ptr()),
        );
        if h.is_invalid() {
            anyhow::bail!("Failed to create font");
        }
        Ok(SafeFont(h))
    }
}

pub fn create_pen(
    style: windows::Win32::Graphics::Gdi::PEN_STYLE,
    width: i32,
    color: u32,
) -> anyhow::Result<SafePen> {
    unsafe {
        let h = windows::Win32::Graphics::Gdi::CreatePen(style, width, COLORREF(color));
        if h.is_invalid() {
            anyhow::bail!("Failed to create pen");
        }
        Ok(SafePen(h))
    }
}

pub fn register_font(path: &std::path::Path) -> anyhow::Result<i32> {
    unsafe {
        let path_str = path.to_string_lossy();
        let path_u16: Vec<u16> = path_str.encode_utf16().chain(Some(0)).collect();
        let count = AddFontResourceExW(windows::core::PCWSTR(path_u16.as_ptr()), FR_PRIVATE, None);
        if count == 0 {
            anyhow::bail!("Failed to register font: {}", path_str);
        }
        Ok(count)
    }
}

pub fn unregister_font(path: &std::path::Path) -> anyhow::Result<()> {
    unsafe {
        let path_str = path.to_string_lossy();
        let path_u16: Vec<u16> = path_str.encode_utf16().chain(Some(0)).collect();
        if RemoveFontResourceExW(windows::core::PCWSTR(path_u16.as_ptr()), FR_PRIVATE.0, None)
            .as_bool()
        {
            Ok(())
        } else {
            anyhow::bail!("Failed to unregister font: {}", path_str);
        }
    }
}

pub fn add_font_mem(data: &[u8]) -> anyhow::Result<windows::Win32::Foundation::HANDLE> {
    unsafe {
        let mut count = 0;
        let handle = windows::Win32::Graphics::Gdi::AddFontMemResourceEx(
            data.as_ptr() as *const std::ffi::c_void,
            data.len() as u32,
            None,
            &mut count,
        );
        if handle.is_invalid() || count == 0 {
            anyhow::bail!("Failed to add font from memory");
        }
        Ok(handle)
    }
}
