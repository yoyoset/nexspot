use parley::layout::PositionedLayoutItem;
use parley::Layout;
use vello::kurbo::Affine;
use vello::peniko::Brush;
use vello::Scene;

pub fn draw_layout_to_scene(
    scene: &mut Scene,
    layout: &Layout<[u8; 4]>,
    transform: Affine,
    brush: &Brush,
) {
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let run = glyph_run.run();
                let font = run.font();
                let font_size = run.font_size();

                // Construct FontData directly, now parley and vello should share the same peniko version.
                let font_data = vello::peniko::FontData::new(font.data.clone(), font.index);

                let mut run_x = glyph_run.offset();
                let run_y = glyph_run.baseline();

                scene
                    .draw_glyphs(&font_data)
                    .font_size(font_size)
                    .transform(transform)
                    .brush(brush)
                    .draw(
                        vello::peniko::Fill::NonZero,
                        glyph_run.glyphs().map(|g| {
                            let gx = run_x + g.x;
                            let gy = run_y - g.y; // Vello uses downward Y, Parley uses upward Y for offsets
                            run_x += g.advance; // Accumulate cursor

                            vello::Glyph {
                                id: g.id as u32,
                                x: gx,
                                y: gy,
                            }
                        }),
                    );
            }
        }
    }
}
