use crate::service::native_overlay::render::toolbar::{ButtonState, Toolbar};
use crate::service::native_overlay::render::vello_engine::renderer::utils;
use crate::service::native_overlay::state::OverlayState;
use vello::kurbo::{Affine, Rect, RoundedRect};
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

pub fn draw_tooltip_ui(
    scene: &mut Scene,
    _state: &OverlayState,
    toolbar: &Toolbar,
    font_ctx: &mut parley::FontContext,
    layout_ctx: &mut parley::LayoutContext<[u8; 4]>,
) {
    let hovered_btn = toolbar
        .main_buttons
        .iter()
        .find(|btn| btn.state == ButtonState::Hover);

    let btn = match hovered_btn {
        Some(b) => b,
        None => return,
    };


    let text = btn.tooltip.clone();
    if text.is_empty() {
        return;
    }

    // 2. Prepare Tooltip Style
    let font_size = 16.0;
    let padding = 12.0;

    // 3. Layout Text using Parley
    let mut builder = layout_ctx.ranged_builder(font_ctx, &text, 1.0, false);
    builder.push_default(parley::style::StyleProperty::FontSize(font_size));
    builder.push_default(parley::style::StyleProperty::Brush([255, 255, 255, 255])); // White brush

    let layout: parley::Layout<[u8; 4]> = {
        let mut layout = builder.build(&text);
        layout.break_all_lines(None);
        layout
    };

    let text_width = layout.width();
    let text_height = layout.height();

    // 4. Calculate Tooltip Position
    let btn_width = btn.rect.right - btn.rect.left;
    let _btn_height = btn.rect.bottom - btn.rect.top;
    let x = btn.rect.left as f64 + (btn_width as f64 - text_width as f64) / 2.0;
    let y = btn.rect.bottom as f64 + 10.0;

    let bg_rect = Rect::new(
        x - padding,
        y - padding + 2.0,
        x + text_width as f64 + padding,
        y + text_height as f64 + padding - 2.0,
    );
    let bg_shape = RoundedRect::from_rect(bg_rect, 6.0);

    // 5. Draw Background
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(40, 40, 40, 240)),
        None,
        &bg_shape,
    );

    // Draw Border
    scene.stroke(
        &vello::kurbo::Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(80, 80, 80, 255)),
        None,
        &bg_shape,
    );

    // 6. Draw Text
    utils::text::draw_layout_to_scene(
        scene,
        &layout,
        Affine::translate((x, y)),
        &Brush::Solid(Color::WHITE),
    );
}
