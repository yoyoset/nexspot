use super::dc::SafeHDC;
use super::resources::SafeBrush;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    Ellipse, FillRect, FrameRect, LineTo, MoveToEx, Rectangle, RoundRect,
};

pub fn frame_rect(hdc: &SafeHDC, rect: &RECT, brush: &SafeBrush) {
    unsafe {
        let _ = FrameRect(hdc.0, rect, brush.0);
    }
}

pub fn fill_rect(hdc: &SafeHDC, rect: &RECT, brush: &SafeBrush) {
    unsafe {
        let _ = FillRect(hdc.0, rect, brush.0);
    }
}

pub fn rectangle(
    hdc: &SafeHDC,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) -> anyhow::Result<()> {
    unsafe {
        if Rectangle(hdc.0, left, top, right, bottom).as_bool() {
            Ok(())
        } else {
            anyhow::bail!("Rectangle failed");
        }
    }
}

pub fn ellipse(hdc: &SafeHDC, left: i32, top: i32, right: i32, bottom: i32) -> anyhow::Result<()> {
    unsafe {
        if Ellipse(hdc.0, left, top, right, bottom).as_bool() {
            Ok(())
        } else {
            anyhow::bail!("Ellipse failed");
        }
    }
}

pub fn round_rect(
    hdc: &SafeHDC,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    width: i32,
    height: i32,
) -> anyhow::Result<()> {
    unsafe {
        if RoundRect(hdc.0, left, top, right, bottom, width, height).as_bool() {
            Ok(())
        } else {
            anyhow::bail!("RoundRect failed");
        }
    }
}

pub fn move_to(hdc: &SafeHDC, x: i32, y: i32) -> anyhow::Result<()> {
    unsafe {
        if MoveToEx(hdc.0, x, y, None).as_bool() {
            Ok(())
        } else {
            anyhow::bail!("MoveToEx failed");
        }
    }
}

pub fn line_to(hdc: &SafeHDC, x: i32, y: i32) -> anyhow::Result<()> {
    unsafe {
        if LineTo(hdc.0, x, y).as_bool() {
            Ok(())
        } else {
            anyhow::bail!("LineTo failed");
        }
    }
}
