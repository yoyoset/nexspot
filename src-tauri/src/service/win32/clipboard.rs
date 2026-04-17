use std::path::Path;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::Graphics::Gdi::{CreateDIBitmap, GetDC, ReleaseDC, BITMAPINFOHEADER, BI_BITFIELDS, CBM_INIT, DIB_RGB_COLORS};

/// Copies an image file (PNG/JPG) to the system clipboard as a CF_BITMAP.
pub fn set_clipboard_image_from_file(path: &Path) -> anyhow::Result<()> {
    // 1. Load image using image crate
    let img = image::open(path)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    unsafe {
        // 2. Open Clipboard
        if OpenClipboard(None).is_ok() {
            let _ = EmptyClipboard();

            // 3. Create BITMAP from RGBA data
            // To do this properly on Windows, we'll create a DIB (Device Independent Bitmap)
            let mut bmih = BITMAPINFOHEADER::default();
            bmih.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmih.biWidth = width as i32;
            bmih.biHeight = -(height as i32); // Top-down
            bmih.biPlanes = 1;
            bmih.biBitCount = 32;
            bmih.biCompression = BI_BITFIELDS.0;
            bmih.biSizeImage = (width * height * 4) as u32;

            let hdc = GetDC(None);
            
            // Create the bitmap
            let h_bitmap = CreateDIBitmap(
                hdc,
                Some(&bmih),
                CBM_INIT as u32,
                Some(rgba.as_raw().as_ptr() as *const _),
                None, // We don't need a palette for 32bpp
                DIB_RGB_COLORS,
            );
            
            let _ = ReleaseDC(None, hdc);

            if !h_bitmap.is_invalid() {
                // 4. Set to Clipboard (CF_BITMAP = 2)
                let _ = SetClipboardData(2, Some(HANDLE(h_bitmap.0 as *mut _)));
            }

            let _ = CloseClipboard();
        } else {
            anyhow::bail!("Failed to open clipboard");
        }
    }

    Ok(())
}
