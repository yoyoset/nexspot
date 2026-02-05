use super::dc::SafeHDC;
use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Gdi::{SetTextColor, TextOutW};

pub fn text_out(hdc: &SafeHDC, x: i32, y: i32, text: &str) -> anyhow::Result<()> {
    unsafe {
        let text_u16: Vec<u16> = text.encode_utf16().collect();
        let res = TextOutW(hdc.0, x, y, &text_u16);
        res.ok()?;
        Ok(())
    }
}

pub fn set_text_color(hdc: &SafeHDC, color: u32) {
    unsafe {
        let _ = SetTextColor(hdc.0, COLORREF(color));
    }
}
