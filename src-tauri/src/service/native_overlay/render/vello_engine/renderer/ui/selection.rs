use crate::service::native_overlay::state::OverlayState;
use vello::kurbo::{Affine, Rect, Stroke};
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

pub fn draw_selection_ui(
    scene: &mut Scene,
    state: &OverlayState,
    sel: &windows::Win32::Foundation::RECT,
) {
    let width = state.width as f64;
    let height = state.height as f64;
    let sl = sel.left as f64;
    let st = sel.top as f64;
    let sr = sel.right as f64;
    let sb = sel.bottom as f64;

    let wl = state.x as f64;
    let wt = state.y as f64;
    let wr = wl + state.width as f64;
    let wb = wt + state.height as f64;

    let dim_brush = Brush::Solid(Color::from_rgba8(0, 0, 0, 160));

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &dim_brush,
        None,
        &Rect::new(wl, wt, wr, st),
    );
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &dim_brush,
        None,
        &Rect::new(wl, sb, wr, wb),
    );
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &dim_brush,
        None,
        &Rect::new(wl, st, sl, sb),
    );
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &dim_brush,
        None,
        &Rect::new(sr, st, wr, sb),
    );

    let border_color = Color::from_rgba8(0, 160, 255, 255);
    let border_brush = Brush::Solid(border_color);
    let border_stroke = Stroke::new(1.0);
    let rect = Rect::new(sl, st, sr, sb);
    scene.stroke(&border_stroke, Affine::IDENTITY, &border_brush, None, &rect);

    let handle_size = 8.0;
    let handles = [
        (sl, st),
        ((sl + sr) / 2.0, st),
        (sr, st),
        (sl, (st + sb) / 2.0),
        (sr, (st + sb) / 2.0),
        (sl, sb),
        ((sl + sr) / 2.0, sb),
        (sr, sb),
    ];

    let white_brush = Brush::Solid(Color::WHITE);
    let handle_stroke = Stroke::new(1.0);

    for (hx, hy) in handles {
        let h_rect = Rect::new(
            hx - handle_size / 2.0,
            hy - handle_size / 2.0,
            hx + handle_size / 2.0,
            hy + handle_size / 2.0,
        );
        scene.fill(Fill::NonZero, Affine::IDENTITY, &white_brush, None, &h_rect);
        scene.stroke(
            &handle_stroke,
            Affine::IDENTITY,
            &border_brush,
            None,
            &h_rect,
        );
    }
}
