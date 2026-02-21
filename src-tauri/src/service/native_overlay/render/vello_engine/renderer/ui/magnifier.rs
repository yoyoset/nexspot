use crate::service::native_overlay::state::{InteractionMode, OverlayState};
use vello::kurbo::{Affine, Circle, Line, Stroke};
use vello::peniko::{Brush, Color, Fill, ImageBrush};
use vello::Scene;

pub fn draw_magnifier(scene: &mut Scene, state: &OverlayState) {
    let is_adjusting = matches!(
        state.interaction_mode,
        InteractionMode::Selecting | InteractionMode::Resizing(_)
    );

    let is_outside = if let Some(sel) = state.selection {
        state.mouse_x < sel.left
            || state.mouse_x > sel.right
            || state.mouse_y < sel.top
            || state.mouse_y > sel.bottom
    } else {
        true
    };

    if !is_adjusting && !is_outside {
        return;
    }

    let mx = state.mouse_x as f64;
    let my = state.mouse_y as f64;
    let radius = 80.0;
    let offset = 40.0;

    let mag_x = if mx + radius + offset + 10.0 > state.width as f64 {
        mx - radius - offset
    } else {
        mx + offset
    };
    let mag_y = if my + radius + offset + 10.0 > state.height as f64 {
        my - radius - offset
    } else {
        my + offset
    };

    let circle = Circle::new((mag_x + radius, mag_y + radius), radius);

    if let Some(bg) = &state.vello.background {
        let zoom = 2.0;
        let transform = Affine::translate((mag_x + radius, mag_y + radius))
            * Affine::scale(zoom)
            * Affine::translate((-mx, -my));

        let brush = Brush::Image(ImageBrush {
            image: bg.clone(),
            sampler: Default::default(),
        });
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &brush,
            Some(transform),
            &circle,
        );
    }
    let cross_brush = Brush::Solid(Color::from_rgba8(255, 255, 255, 200));
    let cross_stroke = Stroke::new(1.5);
    let cross_len = 10.0;
    let cx = mag_x + radius;
    let cy = mag_y + radius;

    scene.stroke(
        &cross_stroke,
        Affine::IDENTITY,
        &cross_brush,
        None,
        &Line::new((cx - cross_len, cy), (cx + cross_len, cy)),
    );
    scene.stroke(
        &cross_stroke,
        Affine::IDENTITY,
        &cross_brush,
        None,
        &Line::new((cx, cy - cross_len), (cx, cy + cross_len)),
    );

    let border_brush = Brush::Solid(Color::from_rgba8(80, 80, 80, 255));
    let border_stroke = Stroke::new(2.5);
    scene.stroke(
        &border_stroke,
        Affine::IDENTITY,
        &border_brush,
        None,
        &circle,
    );
}
