use crate::service::win32::gdi::SafeHBITMAP;
use anyhow::{bail, Context};
use std::path::Path;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Gdi::{
    CreateDIBitmap, GetDC, ReleaseDC, BITMAPINFOHEADER, BI_BITFIELDS, CBM_INIT, DIB_RGB_COLORS,
};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};

struct ClipboardGuard;

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

/// Copies an image file (PNG/JPG) to the system clipboard as a CF_BITMAP.
pub fn set_clipboard_image_from_file(path: &Path) -> anyhow::Result<()> {
    // 1. Load image using image crate
    let img = image::open(path)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    unsafe {
        OpenClipboard(None).context("Failed to open clipboard")?;
        let _guard = ClipboardGuard;
        EmptyClipboard().context("Failed to empty clipboard")?;

        // 3. Create a top-down bitmap from RGBA data.
        let mut bmih = BITMAPINFOHEADER::default();
        bmih.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmih.biWidth = width as i32;
        bmih.biHeight = -(height as i32);
        bmih.biPlanes = 1;
        bmih.biBitCount = 32;
        bmih.biCompression = BI_BITFIELDS.0;
        bmih.biSizeImage = (width * height * 4) as u32;

        let hdc = GetDC(None);
        let h_bitmap = CreateDIBitmap(
            hdc,
            Some(&bmih),
            CBM_INIT as u32,
            Some(rgba.as_raw().as_ptr() as *const _),
            None,
            DIB_RGB_COLORS,
        );
        let _ = ReleaseDC(None, hdc);

        if h_bitmap.is_invalid() {
            bail!("Failed to create clipboard bitmap");
        }

        let safe_bitmap = SafeHBITMAP(h_bitmap);
        SetClipboardData(2, Some(HANDLE(safe_bitmap.0 .0)))
            .context("Failed to set clipboard bitmap")?;
        std::mem::forget(safe_bitmap);
    }

    Ok(())
}
