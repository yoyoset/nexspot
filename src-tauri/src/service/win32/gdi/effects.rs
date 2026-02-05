use super::dc::{create_compatible_dc, select_object, stretch_blt, SafeHDC};
use super::resources::create_compatible_bitmap;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{SetStretchBltMode, COLORONCOLOR};

pub fn pixelate_rect(hdc: &SafeHDC, rect: &RECT, block_size: i32) -> anyhow::Result<()> {
    unsafe {
        let w = rect.right - rect.left;
        let h = rect.bottom - rect.top;
        if w <= 0 || h <= 0 {
            return Ok(());
        }

        let prev_mode = SetStretchBltMode(hdc.0, COLORONCOLOR);

        // 1. Create a tiny memory DC and bitmap for the downscaled version
        let hdc_temp = create_compatible_dc(Some(hdc))?;
        let dw = (w / block_size).max(1);
        let dh = (h / block_size).max(1);
        let hbm_temp = create_compatible_bitmap(hdc, dw, dh)?;
        let prev_hbm = select_object(
            &hdc_temp,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_temp.0 .0),
        )?;

        // 2. Downscale: Backbuffer -> Temp
        let _ = stretch_blt(
            &hdc_temp,
            0,
            0,
            dw,
            dh,
            hdc,
            rect.left,
            rect.top,
            w,
            h,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        );

        // 3. Upscale: Temp -> Backbuffer
        let _ = stretch_blt(
            hdc,
            rect.left,
            rect.top,
            w,
            h,
            &hdc_temp,
            0,
            0,
            dw,
            dh,
            windows::Win32::Graphics::Gdi::SRCCOPY,
        );

        // Cleanup
        select_object(&hdc_temp, prev_hbm)?;
        SetStretchBltMode(
            hdc.0,
            windows::Win32::Graphics::Gdi::STRETCH_BLT_MODE(prev_mode as i32),
        );
        Ok(())
    }
}
