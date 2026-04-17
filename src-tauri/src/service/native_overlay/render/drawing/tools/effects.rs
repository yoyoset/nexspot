use super::super::traits::DrawingToolRenderer;
use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;

pub struct MosaicRenderer;
impl DrawingToolRenderer for MosaicRenderer {
    fn render(
        &self,
        hdc: &SafeHDC,
        graphics: Option<&crate::service::win32::gdiplus::GraphicsWrapper>,
        src_hdc: Option<&SafeHDC>,
        cache: &mut crate::service::win32::gdi::GdiCache,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if let Some(g) = graphics {
            self.render_gdiplus_with_src(g, src_hdc, cache, obj)
        } else {
            let g = crate::service::win32::gdiplus::GraphicsWrapper::new(hdc.0)?;
            self.render_gdiplus_with_src(&g, src_hdc, cache, obj)
        }
    }

    fn render_gdiplus(
        &self,
        _graphics: &crate::service::win32::gdiplus::GraphicsWrapper,
        _cache: &mut crate::service::win32::gdi::GdiCache,
        _obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn render_gdiplus_with_src(
        &self,
        graphics: &crate::service::win32::gdiplus::GraphicsWrapper,
        _src_hdc: Option<&SafeHDC>,
        cache: &mut crate::service::win32::gdi::GdiCache,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        if obj.mosaic_blocks.is_empty() {
            return Ok(());
        }

        // --- PERFORMANCE OPTIMIZATION: USE GDI Native FillRect ---
        let hdc = graphics.get_hdc()?;

        // Map stroke_width to block size
        let block_size = match obj.stroke_width as i32 {
            0..=3 => 6,
            4..=7 => 10,
            _ => 16,
        };
        let block_size_f = block_size as f32;

        // Group rects by color to minimize brush creations
        let mut color_groups: std::collections::HashMap<
            u32,
            Vec<windows::Win32::Foundation::RECT>,
        > = std::collections::HashMap::with_capacity(10);

        for (&(gx, gy), &color_argb) in &obj.mosaic_blocks {
            let bx = (gx as f32 * block_size_f).round() as i32;
            let by = (gy as f32 * block_size_f).round() as i32;

            let rect = windows::Win32::Foundation::RECT {
                left: bx,
                top: by,
                right: bx + block_size,
                bottom: by + block_size,
            };
            color_groups.entry(color_argb).or_default().push(rect);
        }

        for (color_argb, rects) in color_groups {
            let color_ref = crate::service::win32::gdi::to_colorref(color_argb);
            // get_brush expects u32 colorref bits usually, let's check if get_brush(color_ref.0) works
            let brush = cache.get_brush(color_ref.0)?;
            
            unsafe {
                for rect in rects {
                    windows::Win32::Graphics::Gdi::FillRect(hdc, &rect, brush.0);
                }
            }
        }

        graphics.release_hdc(hdc);
        Ok(())
    }
}

