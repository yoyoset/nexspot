use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::Color;
use vello::Scene;
use super::constants::*;

pub fn draw_color_palette(
    scene: &mut Scene,
    offset_x: &mut f64,
    rect: &windows::Win32::Foundation::RECT,
    current_color: u32,
) {
    let colors =
        crate::service::native_overlay::render::toolbar::property_bar::get_palette_colors();
    for color_u32 in colors {
        let color = Color::from_rgba8(
            ((color_u32 >> 16) & 0xFF) as u8,
            ((color_u32 >> 8) & 0xFF) as u8,
            (color_u32 & 0xFF) as u8,
            255,
        );

        let is_selected = color_u32 == current_color;
        let color_y = (rect.top as f64 + rect.bottom as f64) / 2.0;
        let btn_rect = Rect::new(
            *offset_x,
            color_y - COLOR_ITEM_SIZE / 2.0,
            *offset_x + COLOR_ITEM_SIZE,
            color_y + COLOR_ITEM_SIZE / 2.0,
        );

        if is_selected {
            // Industrial selection ring (Zinc-700ish)
            let ring_rect = btn_rect.inset(-2.0);
            let ring = RoundedRect::from_rect(ring_rect, 2.0);
            scene.stroke(
                &Stroke::new(1.0),
                Affine::IDENTITY,
                Color::from_rgba8(63, 63, 70, 180), // Zinc-700
                None,
                &ring,
            );
        }

        let inner = RoundedRect::from_rect(btn_rect, 1.0);
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            color,
            None,
            &inner,
        );

        // Thin border for color definition
        scene.stroke(
            &Stroke::new(1.0),
            Affine::IDENTITY,
            if is_selected {
                Color::from_rgb8(59, 130, 246) // Blue-500
            } else {
                Color::from_rgba8(39, 39, 42, 255) // Zinc-800
            },
            None,
            &inner,
        );

        if is_selected {
            // Professional Checkmark indicator (Simplified for Vello scene)
            let cx = btn_rect.center().x;
            let cy = btn_rect.center().y;
            let check_color = if color_u32 == 0xFFFFFFFF { Color::BLACK } else { Color::WHITE };
            
            // Draw a small 2px square instead of a circle for "Rationalist" feel
            let indicator = Rect::new(cx - 2.0, cy - 2.0, cx + 2.0, cy + 2.0);
            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                check_color,
                None,
                &indicator,
            );
        }

        *offset_x += COLOR_ITEM_SIZE + COLOR_ITEM_GAP;
    }
}
