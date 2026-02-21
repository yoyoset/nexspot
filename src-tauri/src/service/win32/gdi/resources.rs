use super::dc::SafeHDC;

use windows::Win32::Graphics::Gdi::{
    AddFontResourceExW, CreateCompatibleBitmap, CreateFontW, CreateSolidBrush, DeleteObject,
    GetStockObject, RemoveFontResourceExW, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    FR_PRIVATE, GET_STOCK_OBJECT_FLAGS, HBITMAP, HBRUSH, HFONT, HGDIOBJ, HPEN, OUT_DEFAULT_PRECIS,
    VARIABLE_PITCH,
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
        let h = CreateSolidBrush(super::to_colorref(color));
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
        let h = windows::Win32::Graphics::Gdi::CreatePen(style, width, super::to_colorref(color));
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

pub fn get_bitmap_bits(
    hdc: &SafeHDC,
    hbitmap: &SafeHBITMAP,
    width: i32,
    height: i32,
) -> anyhow::Result<Vec<u8>> {
    unsafe {
        let bi = windows::Win32::Graphics::Gdi::BITMAPINFOHEADER {
            biSize: std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // Top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: windows::Win32::Graphics::Gdi::BI_RGB.0,
            ..Default::default()
        };

        let mut info = windows::Win32::Graphics::Gdi::BITMAPINFO {
            bmiHeader: bi,
            ..Default::default()
        };

        // 4 bytes per pixel * width * height
        let size = (width * height * 4) as usize;
        let mut pixels = vec![0u8; size];

        let result = windows::Win32::Graphics::Gdi::GetDIBits(
            hdc.0,
            hbitmap.0,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut std::ffi::c_void),
            &mut info,
            windows::Win32::Graphics::Gdi::DIB_RGB_COLORS,
        );

        if result == 0 {
            anyhow::bail!("GetDIBits failed");
        }

        // Fix Alpha channel? GDI often sets alpha to 0 for 32-bit RGB.
        // We typically want 255 (opaque) for the background screenshot.
        // Or if it's a screenshot, maybe it has valid data?
        // BitBlt from screen usually gives 0xFF for Alpha or 0x00 depending on source.
        // Let's force Alpha to 0xFF to be safe for D2D (otherwise it might be transparent).
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[3] = 255;
        }

        Ok(pixels)
    }
}

pub fn create_bitmap_from_pixels(
    hdc: &SafeHDC,
    width: i32,
    height: i32,
    pixels: &[u8],
) -> anyhow::Result<SafeHBITMAP> {
    unsafe {
        let bi = windows::Win32::Graphics::Gdi::BITMAPINFOHEADER {
            biSize: std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // Top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: windows::Win32::Graphics::Gdi::BI_RGB.0,
            ..Default::default()
        };

        let info = windows::Win32::Graphics::Gdi::BITMAPINFO {
            bmiHeader: bi,
            ..Default::default()
        };

        let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();

        let hbitmap = windows::Win32::Graphics::Gdi::CreateDIBSection(
            Some(hdc.0),
            &info,
            windows::Win32::Graphics::Gdi::DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0,
        )?;

        if hbitmap.is_invalid() || bits_ptr.is_null() {
            anyhow::bail!("CreateDIBSection failed");
        }

        // Copy pixels
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits_ptr as *mut u8, pixels.len());

        Ok(SafeHBITMAP(hbitmap))
    }
}

pub fn get_stock_object(idx: GET_STOCK_OBJECT_FLAGS) -> anyhow::Result<HGDIOBJ> {
    unsafe {
        let h = GetStockObject(idx);
        if h.0.is_null() {
            anyhow::bail!("Failed to get stock object");
        }
        Ok(h)
    }
}
