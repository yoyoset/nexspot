use std::sync::Once;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::GdiPlus::*;

static GDI_PLUS_INIT: Once = Once::new();
static mut GDI_PLUS_TOKEN: usize = 0;

pub fn init_gdiplus() {
    GDI_PLUS_INIT.call_once(|| {
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            ..Default::default()
        };
        let mut token: usize = 0;
        let mut output = GdiplusStartupOutput::default();
        unsafe {
            // Use pointers for GdiplusStartup to avoid ambiguity
            let _ = GdiplusStartup(&mut token, &input, &mut output as *mut _);
            GDI_PLUS_TOKEN = token;
        }
    });
}

pub struct GraphicsWrapper(pub *mut GpGraphics);

impl GraphicsWrapper {
    pub fn new(hdc: HDC) -> anyhow::Result<Self> {
        init_gdiplus();
        let mut graphics = std::ptr::null_mut();
        unsafe {
            let status = GdipCreateFromHDC(hdc, &mut graphics);
            if status != Ok {
                anyhow::bail!("GdipCreateFromHDC failed: {:?}", status);
            }
            // Enable Anti-Aliasing
            let _ = GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias);
        }
        std::result::Result::Ok(Self(graphics))
    }
}

impl Drop for GraphicsWrapper {
    fn drop(&mut self) {
        unsafe {
            let _ = GdipDeleteGraphics(self.0);
        }
    }
}

pub struct PenWrapper(pub *mut GpPen);

impl PenWrapper {
    pub fn new(color: u32, width: f32) -> anyhow::Result<Self> {
        let mut pen = std::ptr::null_mut();
        unsafe {
            let status = GdipCreatePen1(color, width, UnitPixel, &mut pen);
            if status != Ok {
                anyhow::bail!("GdipCreatePen1 failed: {:?}", status);
            }
        }
        std::result::Result::Ok(Self(pen))
    }
}

impl Drop for PenWrapper {
    fn drop(&mut self) {
        unsafe {
            let _ = GdipDeletePen(self.0);
        }
    }
}

pub struct BrushWrapper(pub *mut GpBrush);

impl BrushWrapper {
    pub fn new_solid(color: u32) -> anyhow::Result<Self> {
        unsafe {
            let mut solid_fill = std::ptr::null_mut();
            let status = GdipCreateSolidFill(color, &mut solid_fill);
            if status != Ok {
                anyhow::bail!("GdipCreateSolidFill failed: {:?}", status);
            }
            std::result::Result::Ok(Self(solid_fill as *mut GpBrush))
        }
    }
}

impl Drop for BrushWrapper {
    fn drop(&mut self) {
        unsafe {
            let _ = GdipDeleteBrush(self.0);
        }
    }
}

pub fn draw_polygon(
    graphics: &GraphicsWrapper,
    brush: &BrushWrapper,
    points: &[(f32, f32)],
) -> anyhow::Result<()> {
    let gdi_points: Vec<PointF> = points
        .iter()
        .map(|(x, y)| PointF { X: *x, Y: *y })
        .collect();
    unsafe {
        let status = GdipFillPolygon(
            graphics.0,
            brush.0,
            gdi_points.as_ptr(),
            gdi_points.len() as i32,
            FillModeAlternate,
        );
        if status != Ok {
            anyhow::bail!("GdipFillPolygon failed: {:?}", status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_line(
    graphics: &GraphicsWrapper,
    pen: &PenWrapper,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> anyhow::Result<()> {
    unsafe {
        let status = GdipDrawLine(graphics.0, pen.0, x1, y1, x2, y2);
        if status != Ok {
            anyhow::bail!("GdipDrawLine failed: {:?}", status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_lines(
    graphics: &GraphicsWrapper,
    pen: &PenWrapper,
    points: &[(f32, f32)],
) -> anyhow::Result<()> {
    let gdi_points: Vec<PointF> = points.iter().map(|&(x, y)| PointF { X: x, Y: y }).collect();
    unsafe {
        let status = GdipDrawLines(
            graphics.0,
            pen.0,
            gdi_points.as_ptr(),
            gdi_points.len() as i32,
        );
        if status != Ok {
            anyhow::bail!("GdipDrawLines failed: {:?}", status);
        }
    }
    std::result::Result::Ok(())
}

pub fn fill_ellipse(
    graphics: &GraphicsWrapper,
    brush: &BrushWrapper,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> anyhow::Result<()> {
    unsafe {
        let status = GdipFillEllipse(graphics.0, brush.0, x, y, width, height);
        if status != Ok {
            anyhow::bail!("GdipFillEllipse failed: {:?}", status);
        }
    }
    std::result::Result::Ok(())
}

pub fn fill_rectangle(
    graphics: &GraphicsWrapper,
    brush: &BrushWrapper,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> anyhow::Result<()> {
    unsafe {
        let status = GdipFillRectangle(graphics.0, brush.0, x, y, width, height);
        if status != Ok {
            anyhow::bail!("GdipFillRectangle failed: {:?}", status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_rectangle(
    graphics: &GraphicsWrapper,
    pen: &PenWrapper,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> anyhow::Result<()> {
    unsafe {
        let status = GdipDrawRectangle(graphics.0, pen.0, x, y, width, height);
        if status != Ok {
            anyhow::bail!("GdipDrawRectangle failed: {:?}", status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_ellipse(
    graphics: &GraphicsWrapper,
    pen: &PenWrapper,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> anyhow::Result<()> {
    unsafe {
        let status = GdipDrawEllipse(graphics.0, pen.0, x, y, width, height);
        if status != Ok {
            anyhow::bail!("GdipDrawEllipse failed: {:?}", status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_text(
    graphics: &GraphicsWrapper,
    text: &str,
    pos: (f32, f32),
    font_family: &str,
    font_size: f32,
    brush: &BrushWrapper,
    font_style: Option<FontStyle>,
    _layout: Option<RectF>,
) -> anyhow::Result<()> {
    use windows::core::{HSTRING, PCWSTR};
    let text_h = HSTRING::from(text);
    let style = font_style.unwrap_or(FontStyleRegular);

    unsafe {
        let mut family = std::ptr::null_mut();
        let font_family_h = HSTRING::from(font_family);
        let status = GdipCreateFontFamilyFromName(
            PCWSTR(font_family_h.as_ptr()),
            std::ptr::null_mut(),
            &mut family,
        );
        if status != Ok {
            anyhow::bail!("GdipCreateFontFamilyFromName failed: {:?}", status);
        }

        let mut font = std::ptr::null_mut();
        let status = GdipCreateFont(family, font_size, style.0, UnitPixel, &mut font);
        if status != Ok {
            let _ = GdipDeleteFontFamily(family);
            anyhow::bail!("GdipCreateFont failed: {:?}", status);
        }

        let layout_rect = RectF {
            X: pos.0,
            Y: pos.1,
            Width: 0.0,
            Height: 0.0,
        };

        let status = GdipDrawString(
            graphics.0,
            PCWSTR(text_h.as_ptr()),
            text_h.len() as i32,
            font,
            &layout_rect,
            std::ptr::null(),
            brush.0,
        );

        let _ = GdipDeleteFont(font);
        let _ = GdipDeleteFontFamily(family);

        if status != Ok {
            anyhow::bail!("GdipDrawString failed: {:?}", status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_text_centered(
    graphics: &GraphicsWrapper,
    text: &str,
    center: (f32, f32),
    font_family: &str,
    font_size: f32,
    brush: &BrushWrapper,
    font_style: Option<FontStyle>,
) -> anyhow::Result<()> {
    use windows::core::{HSTRING, PCWSTR};
    let text_h = HSTRING::from(text);
    let style = font_style.unwrap_or(FontStyleRegular);

    unsafe {
        let mut family = std::ptr::null_mut();
        let font_family_h = HSTRING::from(font_family);
        let status = GdipCreateFontFamilyFromName(
            PCWSTR(font_family_h.as_ptr()),
            std::ptr::null_mut(),
            &mut family,
        );
        if status != Ok {
            anyhow::bail!("GdipCreateFontFamilyFromName failed: {:?}", status);
        }

        let mut font = std::ptr::null_mut();
        let status = GdipCreateFont(family, font_size, style.0, UnitPixel, &mut font);
        if status != Ok {
            let _ = GdipDeleteFontFamily(family);
            anyhow::bail!("GdipCreateFont failed: {:?}", status);
        }

        let mut format = std::ptr::null_mut();
        let _ = GdipCreateStringFormat(0, 0, &mut format);
        let _ = GdipSetStringFormatAlign(format, StringAlignmentCenter);
        let _ = GdipSetStringFormatLineAlign(format, StringAlignmentCenter);

        let layout_rect = RectF {
            X: center.0,
            Y: center.1,
            Width: 0.0,
            Height: 0.0,
        };

        let status = GdipDrawString(
            graphics.0,
            PCWSTR(text_h.as_ptr()),
            text_h.len() as i32,
            font,
            &layout_rect,
            format,
            brush.0,
        );

        let _ = GdipDeleteStringFormat(format);
        let _ = GdipDeleteFont(font);
        let _ = GdipDeleteFontFamily(family);

        if status != Ok {
            anyhow::bail!("GdipDrawString failed: {:?}", status);
        }
    }
    std::result::Result::Ok(())
}

pub fn measure_text(
    graphics: &GraphicsWrapper,
    text: &str,
    font_family: &str,
    font_size: f32,
    font_style: Option<FontStyle>,
) -> anyhow::Result<RectF> {
    use windows::core::{HSTRING, PCWSTR};
    let text_h = HSTRING::from(text);
    let style = font_style.unwrap_or(FontStyleRegular);

    unsafe {
        let mut family = std::ptr::null_mut();
        let font_family_h = HSTRING::from(font_family);
        let status = GdipCreateFontFamilyFromName(
            PCWSTR(font_family_h.as_ptr()),
            std::ptr::null_mut(),
            &mut family,
        );
        if status != Ok {
            anyhow::bail!("GdipCreateFontFamilyFromName failed: {:?}", status);
        }

        let mut font = std::ptr::null_mut();
        let status = GdipCreateFont(family, font_size, style.0, UnitPixel, &mut font);
        if status != Ok {
            let _ = GdipDeleteFontFamily(family);
            anyhow::bail!("GdipCreateFont failed: {:?}", status);
        }

        let layout_rect = RectF {
            X: 0.0,
            Y: 0.0,
            Width: 0.0,
            Height: 0.0,
        };
        let mut bounding_box = RectF::default();

        let status = GdipMeasureString(
            graphics.0,
            PCWSTR(text_h.as_ptr()),
            text_h.len() as i32,
            font,
            &layout_rect,
            std::ptr::null(),
            &mut bounding_box,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        let _ = GdipDeleteFont(font);
        let _ = GdipDeleteFontFamily(family);

        if status != Ok {
            anyhow::bail!("GdipMeasureString failed: {:?}", status);
        }
        std::result::Result::Ok(bounding_box)
    }
}
