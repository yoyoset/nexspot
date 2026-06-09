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
            // Studio accent ring：inset 正值=外扩（kurbo 语义），画在色块外侧才可见
            let ring_rect = btn_rect.inset(2.5);
            let ring = RoundedRect::from_rect(ring_rect, 7.0);
            scene.stroke(
                &Stroke::new(1.5),
                Affine::IDENTITY,
                Color::from_rgba8(122, 111, 242, 255), // --accent
                None,
                &ring,
            );
        }

        let inner = RoundedRect::from_rect(btn_rect, 5.0);
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
            Color::from_rgba8(255, 255, 255, 24), // 微弱白描边，统一明暗
            None,
            &inner,
        );

        *offset_x += COLOR_ITEM_SIZE + COLOR_ITEM_GAP;
    }
}
