use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, DeleteDC, GetPixel, SelectObject, SetBkMode, StretchBlt,
    BACKGROUND_MODE, HDC, HGDIOBJ, ROP_CODE,
};

#[derive(Debug)]
pub enum Disposer {
    Delete,
    Release(Option<windows::Win32::Foundation::HWND>),
    None,
}

impl std::fmt::Debug for SafeHDC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SafeHDC")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

pub struct SafeHDC(pub(crate) HDC, pub(crate) Disposer);

impl Drop for SafeHDC {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                match &self.1 {
                    Disposer::Delete => {
                        let _ = DeleteDC(self.0);
                    }
                    Disposer::Release(hwnd) => {
                        windows::Win32::Graphics::Gdi::ReleaseDC(*hwnd, self.0);
                    }
                    Disposer::None => {}
                }
            }
        }
    }
}

unsafe impl Send for SafeHDC {}
unsafe impl Sync for SafeHDC {}

pub fn create_compatible_dc(hdc: Option<&SafeHDC>) -> anyhow::Result<SafeHDC> {
    unsafe {
        let h = CreateCompatibleDC(hdc.map(|h| h.0));
        if h.is_invalid() {
            anyhow::bail!("Failed to create compatible DC");
        }
        Ok(SafeHDC(h, Disposer::Delete))
    }
}

pub fn select_object(hdc: &SafeHDC, obj: HGDIOBJ) -> anyhow::Result<HGDIOBJ> {
    unsafe {
        let prev = SelectObject(hdc.0, obj);
        Ok(prev)
    }
}

pub fn get_pixel(hdc: &SafeHDC, x: i32, y: i32) -> u32 {
    unsafe { GetPixel(hdc.0, x, y).0 }
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
        Ok(SafeHDC(h, Disposer::Release(hwnd)))
    }
}

pub fn release_dc(_hwnd: Option<windows::Win32::Foundation::HWND>, hdc: SafeHDC) {
    unsafe {
        match hdc.1 {
            Disposer::Release(h) => {
                windows::Win32::Graphics::Gdi::ReleaseDC(h, hdc.0);
            }
            Disposer::Delete => {
                let _ = windows::Win32::Graphics::Gdi::DeleteDC(hdc.0);
            }
            Disposer::None => {}
        }
        // Prevent double drop
        std::mem::forget(hdc);
    }
}

pub fn create_display_dc(name: &str) -> anyhow::Result<SafeHDC> {
    use windows::core::w;
    let name_u16: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let h = windows::Win32::Graphics::Gdi::CreateDCW(
            w!("DISPLAY"),
            windows::core::PCWSTR(name_u16.as_ptr()),
            None,
            None,
        );
        if h.is_invalid() {
            anyhow::bail!("Failed to create display DC for {}", name);
        }
        Ok(SafeHDC(h, Disposer::Delete))
    }
}
