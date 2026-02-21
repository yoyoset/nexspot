use std::ffi::c_void;
use std::path::Path;
use windows::Win32::Graphics::Gdi::{
    GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};

pub fn save_bitmap_to_file(
    hbitmap: windows::Win32::Graphics::Gdi::HBITMAP,
    path: &Path,
    quality: u8,
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
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png")
            .to_lowercase();

        if extension == "jpg" || extension == "jpeg" {
            // JPEG needs RGB, so we convert BGRA to RGB
            let mut rgb_pixels = Vec::with_capacity((width * height * 3) as usize);
            for chunk in pixels.chunks_exact(4) {
                rgb_pixels.push(chunk[2]); // R
                rgb_pixels.push(chunk[1]); // G
                rgb_pixels.push(chunk[0]); // B
            }

            let mut file = std::fs::File::create(path)?;
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
            encoder.encode_image(
                &image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                    width as u32,
                    height as u32,
                    rgb_pixels,
                )
                .unwrap(),
            )?;
        } else {
            // Manual swap BGRA -> RGBA
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
        }

        Ok(())
    }
}

pub fn bitmap_to_bytes(
    hbitmap: windows::Win32::Graphics::Gdi::HBITMAP,
    format: image::ImageFormat,
    quality: u8,
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

        // 4. Encode
        let mut buffer = std::io::Cursor::new(Vec::new());
        if format == image::ImageFormat::Jpeg {
            let mut rgb_pixels = Vec::with_capacity((width * height * 3) as usize);
            for chunk in pixels.chunks_exact(4) {
                rgb_pixels.push(chunk[2]); // R
                rgb_pixels.push(chunk[1]); // G
                rgb_pixels.push(chunk[0]); // B
            }
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
            encoder.encode_image(
                &image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                    width as u32,
                    height as u32,
                    rgb_pixels,
                )
                .unwrap(),
            )?;
        } else {
            // Manual swap BGRA -> RGBA
            for chunk in pixels.chunks_exact_mut(4) {
                let b = chunk[0];
                let r = chunk[2];
                chunk[0] = r;
                chunk[2] = b;
            }

            image::write_buffer_with_format(
                &mut buffer,
                &pixels,
                width as u32,
                height as u32,
                image::ColorType::Rgba8,
                format,
            )?;
        }

        Ok(buffer.into_inner())
    }
}
