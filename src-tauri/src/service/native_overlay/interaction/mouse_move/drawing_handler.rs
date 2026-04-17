use crate::service::native_overlay::state::{DrawingTool, OverlayState};
use windows::Win32::Foundation::RECT;

pub fn handle(state: &mut OverlayState, x: i32, y: i32) {
    if let Some(drawing) = &mut state.current_drawing {
        let rect = state.selection.unwrap_or(RECT {
            left: 0,
            top: 0,
            right: state.width,
            bottom: state.height,
        });

        let cx = x.clamp(rect.left, rect.right);
        let cy = y.clamp(rect.top, rect.bottom);

        match drawing.tool {
            DrawingTool::Brush => {
                let last = drawing.points.last().copied().unwrap_or((0, 0));
                if (cx - last.0).abs() > 2 || (cy - last.1).abs() > 2 {
                    drawing.points.push((cx, cy));
                }
            }
            DrawingTool::Mosaic => {
                let last = drawing.points.last().copied().unwrap_or((cx, cy));
                if drawing.points.is_empty() {
                    drawing.points.push((cx, cy));
                    drawing.mosaic_pending_points.push_back((cx, cy));
                } else if (cx - last.0).abs() > 2 || (cy - last.1).abs() > 2 {
                    drawing.points.push((cx, cy));
                    drawing.mosaic_pending_points.push_back((cx, cy));
                }

                // Process with budget (50 points per tick)
                let vello_bg = state.vello.background.clone();
                let gdi_hbm = state.gdi.hbitmap_bright.as_ref();
                let gdi_pixels = state.gdi.bright_pixels.clone();

                let offset_x = state.capture_x as f64;
                let offset_y = state.capture_y as f64;

                drawing.process_mosaic_pending_points(50, |x, y| {
                    if let Some(img) = &vello_bg {
                        sample_mosaic_vello_fast(img, x - offset_x, y - offset_y)
                    } else if let Some(hbm) = &gdi_hbm {
                        sample_mosaic_gdi_fast(
                            hbm,
                            gdi_pixels.as_ref().map(|p| p.as_ref()),
                            state.width,
                            x - offset_x,
                            y - offset_y,
                        )
                    } else {
                        0xFF808080
                    }
                });
            }
            _ => {
                if drawing.points.len() == 1 {
                    drawing.points.push((cx, cy));
                } else if drawing.points.len() == 2 {
                    drawing.points[1] = (cx, cy);
                }
            }
        }
    }

    // Also process ANY selected object that might be lagging
    if let Some(idx) = state.selected_object_index {
        if let Some(obj) = state.objects.get_mut(idx) {
            if obj.tool == DrawingTool::Mosaic && !obj.mosaic_pending_points.is_empty() {
                let vello_bg = state.vello.background.clone();
                let gdi_hbm = state.gdi.hbitmap_bright.as_ref();
                let gdi_pixels = state.gdi.bright_pixels.clone();

                let offset_x = state.capture_x as f64;
                let offset_y = state.capture_y as f64;

                obj.process_mosaic_pending_points(50, |x, y| {
                    if let Some(img) = &vello_bg {
                        sample_mosaic_vello_fast(img, x - offset_x, y - offset_y)
                    } else if let Some(hbm) = &gdi_hbm {
                        sample_mosaic_gdi_fast(
                            hbm,
                            gdi_pixels.as_ref().map(|p| p.as_ref()),
                            state.width,
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

fn sample_mosaic_vello_fast(img: &vello::peniko::ImageData, x: f64, y: f64) -> u32 {
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

fn sample_mosaic_gdi_fast(
    hbm: &crate::service::win32::gdi::SafeHBITMAP,
    pixels: Option<&Vec<u32>>,
    width: i32,
    x: f64,
    y: f64,
) -> u32 {
    let ix = x.round() as i32;
    let iy = y.round() as i32;

    if let Some(data) = pixels {
        if ix >= 0 && ix < width && iy >= 0 {
            let idx = (iy * width + ix) as usize;
            if idx < data.len() {
                return data[idx];
            }
        }
    }

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
