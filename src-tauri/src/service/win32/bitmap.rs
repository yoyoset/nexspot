use std::ffi::c_void;
use std::path::Path;
use windows::Win32::Graphics::Gdi::{
    GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};

pub fn save_bitmap_to_file(
    hbitmap: windows::Win32::Graphics::Gdi::HBITMAP,
    path: &Path,
) -> anyhow::Result<()> {
    unsafe {
        // 1. Get BITMAP info
        let mut bmp = BITMAP::default();
        let get_obj_res = GetObjectW(
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbitmap.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bmp as *mut _ as *mut c_void),
        );

        if get_obj_res == 0 {
            anyhow::bail!("Failed to get bitmap object");
        }

        let width = bmp.bmWidth;
        let height = bmp.bmHeight;

        // 2. Prepare BITMAPINFO
        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        // 3. Get Bits
        // We need a DC to call GetDIBits. Any screen DC works.
        let hdc = windows::Win32::Graphics::Gdi::GetDC(None);
        if hdc.is_invalid() {
            anyhow::bail!("Failed to get DC for saving");
        }

        let mut pixels: Vec<u8> = vec![0; (width * height * 4) as usize];

        let lines = GetDIBits(
            hdc,
            hbitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut bi,
            DIB_RGB_COLORS,
        );

        windows::Win32::Graphics::Gdi::ReleaseDC(None, hdc);

        if lines == 0 {
            anyhow::bail!("Failed to get DI bits");
        }

        // 4. Save using image crate
        // Pixels are BGRA (Windows default for 32-bit), image::save_buffer expects RGBA or we can save as BGRA if supported?
        // image crate `save_buffer` with ColorType::Rgba8 expects R-G-B-A.
        // Windows returns B-G-R-A likely.
        // Let's swap generic BGRA -> RGBA.

        // Parallel swap if possible, or simple loop
        for chunk in pixels.chunks_exact_mut(4) {
            let b = chunk[0];
            let r = chunk[2];
            chunk[0] = r;
            chunk[2] = b;
        }

        image::save_buffer(
            path,
            &pixels,
            width as u32,
            height as u32,
            image::ColorType::Rgba8,
        )?;

        Ok(())
    }
}

pub fn bitmap_to_png_bytes(
    hbitmap: windows::Win32::Graphics::Gdi::HBITMAP,
) -> anyhow::Result<Vec<u8>> {
    unsafe {
        // 1. Get BITMAP info
        let mut bmp = BITMAP::default();
        let get_obj_res = GetObjectW(
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbitmap.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bmp as *mut _ as *mut c_void),
        );

        if get_obj_res == 0 {
            anyhow::bail!("Failed to get bitmap object");
        }

        let width = bmp.bmWidth;
        let height = bmp.bmHeight;

        // 2. Prepare BITMAPINFO
        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        // 3. Get Bits
        let hdc = windows::Win32::Graphics::Gdi::GetDC(None);
        if hdc.is_invalid() {
            anyhow::bail!("Failed to get DC for retrieval");
        }

        let mut pixels: Vec<u8> = vec![0; (width * height * 4) as usize];
        let lines = GetDIBits(
            hdc,
            hbitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut bi,
            DIB_RGB_COLORS,
        );

        windows::Win32::Graphics::Gdi::ReleaseDC(None, hdc);

        if lines == 0 {
            anyhow::bail!("Failed to get DI bits");
        }

        // 4. BGRA -> RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            let b = chunk[0];
            let r = chunk[2];
            chunk[0] = r;
            chunk[2] = b;
        }

        // 5. Encode as PNG
        let mut buffer = std::io::Cursor::new(Vec::new());
        image::write_buffer_with_format(
            &mut buffer,
            &pixels,
            width as u32,
            height as u32,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )?;

        Ok(buffer.into_inner())
    }
}
