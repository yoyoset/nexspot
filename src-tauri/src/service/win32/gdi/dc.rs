use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, DeleteDC, SelectObject, SetBkMode, StretchBlt, BACKGROUND_MODE,
    HDC, HGDIOBJ, ROP_CODE,
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

pub fn create_compatible_dc(hdc: Option<&SafeHDC>) -> anyhow::Result<SafeHDC> {
    unsafe {
        let h = CreateCompatibleDC(hdc.map(|h| h.0));
        if h.is_invalid() {
            anyhow::bail!("Failed to create compatible DC");
        }
        Ok(SafeHDC(h))
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
        let res = StretchBlt(dest.0, dx, dy, dw, dh, Some(src.0), sx, sy, sw, sh, rop);
        res.ok()?;
        Ok(())
    }
}

pub fn set_bk_mode(hdc: &SafeHDC, mode: BACKGROUND_MODE) {
    unsafe {
        let _ = SetBkMode(hdc.0, mode);
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
