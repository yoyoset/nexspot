use super::OverlayState;
use crate::service::native_overlay::state::types::{DrawingTool, PropertyChange};
use vello::peniko::ImageData;

impl OverlayState {
    pub fn apply_property_change(&mut self, change: PropertyChange) {
        match change {
            PropertyChange::Color(color) => {
                self.current_color = color;
                if let Some(drawing) = self.objects.iter_mut().rev().find(|o| o.is_editing_text) {
                    drawing.color = color;
                }
            }
            PropertyChange::FontSize(size) => {
                self.current_font_size = size;
                if let Some(drawing) = self.objects.iter_mut().rev().find(|o| o.is_editing_text) {
                    drawing.font_size = size;
                }
            }
            PropertyChange::Stroke(stroke) => {
                self.current_stroke = stroke;
            }
            PropertyChange::Fill(_) => {
                self.current_is_filled = !self.current_is_filled;
            }
            PropertyChange::Opacity(opacity) => {
                self.current_opacity = opacity;
                if let Some(idx) = self.selected_object_index {
                    if let Some(obj) = self.objects.get_mut(idx) {
                        obj.opacity = opacity;
                    }
                }
            }
            PropertyChange::Shadow(enabled) => {
                self.current_shadow = enabled;
                if let Some(idx) = self.selected_object_index {
                    if let Some(obj) = self.objects.get_mut(idx) {
                        obj.has_shadow = enabled;
                    }
                }
            }
            PropertyChange::Glow(glow) => {
                self.current_glow = glow;
                if let Some(idx) = self.selected_object_index {
                    if let Some(obj) = self.objects.get_mut(idx) {
                        obj.glow = glow;
                    }
                }
            }
            PropertyChange::Style(style) => {
                self.current_style = style;
                if let Some(idx) = self.selected_object_index {
                    if let Some(obj) = self.objects.get_mut(idx) {
                        obj.style = style;
                    }
                }
            }
        }
    }

    pub fn undo(&mut self) {
        self.objects.pop();
        // Clear selection if the selected object was removed or if we just want to clear it after undo
        self.selected_object_index = None;
    }

    pub fn finalize_all_objects(&mut self) {
        // 1. Process current drawing (Finalize Interaction)
        if let Some(drawing) = self.current_drawing.take() {
            // Only commit if it has content (2+ points or special tool)
            if drawing.points.len() >= 2
                || matches!(drawing.tool, DrawingTool::Number)
                || matches!(drawing.tool, DrawingTool::Text)
            {
                self.objects.push(drawing);
            }
        }

        let vello_bg = self.vello.background.clone();
        // Borrow the handle instead of cloning
        let gdi_hbm = self.gdi.hbitmap_bright.as_ref();

        // 2. Process all objects (including the one just pushed) for Mosaic
        for obj in &mut self.objects {
            if obj.tool == DrawingTool::Mosaic && !obj.mosaic_pending_points.is_empty() {
                let offset_x = self.capture_x as f64;
                let offset_y = self.capture_y as f64;
                let gdi_pixels = self.gdi.bright_pixels.clone();
                let width = self.width;

                obj.process_mosaic_pending_points(usize::MAX, |x, y| {
                    if let Some(img) = &vello_bg {
                        sample_mosaic_color_vello_final(img, x - offset_x, y - offset_y)
                    } else if let Some(hbm) = gdi_hbm {
                        sample_mosaic_color_gdi_final(
                            hbm,
                            gdi_pixels.as_deref().map(|p| p.as_slice()),
                            width,
                            x - offset_x,
                            y - offset_y,
                        )
                    } else {
                        0xFF808080
                    }
                });
            }
        }
    }
}

fn sample_mosaic_color_vello_final(img: &ImageData, x: f64, y: f64) -> u32 {
    let ix = x.round() as i32;
    let iy = y.round() as i32;
    if ix >= 0 && ix < img.width as i32 && iy >= 0 && iy < img.height as i32 {
        let idx = (iy * img.width as i32 + ix) as usize * 4;
        let data = img.data.as_ref();
        if idx + 3 < data.len() {
            let r = data[idx];
            let g = data[idx + 1];
            let b = data[idx + 2];
            let a = data[idx + 3];
            return ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        }
    }
    0xFF808080
}

fn sample_mosaic_color_gdi_final(
    hbm: &crate::service::win32::gdi::SafeHBITMAP,
    pixels: Option<&[u32]>,
    width: i32,
    x: f64,
    y: f64,
) -> u32 {
    let ix = x.round() as i32;
    let iy = y.round() as i32;

    // RAM cache path
    if let Some(data) = pixels {
        if ix >= 0 && ix < width && iy >= 0 {
            let idx = (iy * width + ix) as usize;
            if idx < data.len() {
                return data[idx];
            }
        }
    }

    // GDI fallback
    if let Ok(hdc_screen) = crate::service::win32::gdi::get_dc(None) {
        if let Ok(hdc_mem) = crate::service::win32::gdi::create_compatible_dc(Some(&hdc_screen)) {
            if let Ok(old) = crate::service::win32::gdi::select_object(
                &hdc_mem,
                windows::Win32::Graphics::Gdi::HGDIOBJ(hbm.0 .0),
            ) {
                let color_ref = crate::service::win32::gdi::get_pixel(&hdc_mem, ix, iy);
                let _ = crate::service::win32::gdi::select_object(&hdc_mem, old);
                let r = color_ref & 0x000000FF;
                let g = (color_ref & 0x0000FF00) >> 8;
                let b = (color_ref & 0x00FF0000) >> 16;
                return 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    }
    0xFF808080
}
