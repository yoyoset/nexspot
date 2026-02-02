use windows::core::BOOL;
use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush, DeleteDC,
    DeleteObject, FillRect, FrameRect, SelectObject, SetBkMode, SetTextColor, StretchBlt, TextOutW,
    BACKGROUND_MODE, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, HBITMAP, HBRUSH, HDC,
    HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, ROP_CODE, VARIABLE_PITCH,
};

pub struct SafeHDC(pub(crate) HDC);

impl Drop for SafeHDC {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DeleteDC(self.0);
            }
        }
    }
}

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

pub fn create_compatible_dc(hdc: Option<&SafeHDC>) -> anyhow::Result<SafeHDC> {
    unsafe {
        let h = CreateCompatibleDC(hdc.map(|h| h.0));
        if h.is_invalid() {
            anyhow::bail!("Failed to create compatible DC");
        }
        Ok(SafeHDC(h))
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

pub fn select_object(hdc: &SafeHDC, obj: HGDIOBJ) -> anyhow::Result<HGDIOBJ> {
    unsafe {
        let prev = SelectObject(hdc.0, obj);
        Ok(prev)
    }
}

pub fn bit_blt(
    dest: &SafeHDC,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    src: &SafeHDC,
    sx: i32,
    sy: i32,
    rop: ROP_CODE,
) -> anyhow::Result<()> {
    unsafe {
        // In windows-rs 0.61.3, BitBlt returns Result
        BitBlt(dest.0, x, y, w, h, Some(src.0), sx, sy, rop)?;
        Ok(())
    }
}

pub fn stretch_blt(
    dest: &SafeHDC,
    dx: i32,
    dy: i32,
    dw: i32,
    dh: i32,
    src: &SafeHDC,
    sx: i32,
    sy: i32,
    sw: i32,
    sh: i32,
    rop: ROP_CODE,
) -> anyhow::Result<()> {
    unsafe {
        // StretchBlt often still returns BOOL
        let res: BOOL = StretchBlt(dest.0, dx, dy, dw, dh, Some(src.0), sx, sy, sw, sh, rop);
        res.ok()?;
        Ok(())
    }
}

pub fn frame_rect(hdc: &SafeHDC, rect: &RECT, brush: &SafeBrush) {
    unsafe {
        FrameRect(hdc.0, rect, brush.0);
    }
}

pub fn fill_rect(hdc: &SafeHDC, rect: &RECT, brush: &SafeBrush) {
    unsafe {
        FillRect(hdc.0, rect, brush.0);
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

pub fn text_out(hdc: &SafeHDC, x: i32, y: i32, text: &str) -> anyhow::Result<()> {
    unsafe {
        let text_u16: Vec<u16> = text.encode_utf16().collect();
        // TextOutW returns BOOL
        let res: BOOL = TextOutW(hdc.0, x, y, &text_u16);
        res.ok()?;
        Ok(())
    }
}

pub fn set_bk_mode(hdc: &SafeHDC, mode: BACKGROUND_MODE) {
    unsafe {
        let _ = SetBkMode(hdc.0, mode);
    }
}

pub fn set_text_color(hdc: &SafeHDC, color: u32) {
    unsafe {
        let _ = SetTextColor(hdc.0, COLORREF(color));
    }
}
pub fn get_dc(hwnd: Option<windows::Win32::Foundation::HWND>) -> anyhow::Result<SafeHDC> {
    unsafe {
        let h = windows::Win32::Graphics::Gdi::GetDC(hwnd);
        if h.is_invalid() {
            anyhow::bail!("Failed to get DC");
        }
        Ok(SafeHDC(h))
    }
}

pub fn release_dc(hwnd: Option<windows::Win32::Foundation::HWND>, hdc: SafeHDC) {
    unsafe {
        windows::Win32::Graphics::Gdi::ReleaseDC(hwnd, hdc.0);
        // Prevent double drop
        std::mem::forget(hdc);
    }
}
