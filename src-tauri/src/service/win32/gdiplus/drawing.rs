use super::wrappers::{BrushWrapper, GraphicsWrapper, PenWrapper};
use windows::Win32::Graphics::GdiPlus::*;

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
        let _status = GdipFillPolygon(
            graphics.0,
            brush.0,
            gdi_points.as_ptr(),
            gdi_points.len() as i32,
            FillModeAlternate,
        );
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipFillPolygon failed: {:?}", _status);
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
        let _status = GdipDrawLine(graphics.0, pen.0, x1, y1, x2, y2);
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipDrawLine failed: {:?}", _status);
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
        let _status = GdipDrawLines(
            graphics.0,
            pen.0,
            gdi_points.as_ptr(),
            gdi_points.len() as i32,
        );
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipDrawLines failed: {:?}", _status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_curve(
    graphics: &GraphicsWrapper,
    pen: &PenWrapper,
    points: &[(f32, f32)],
) -> anyhow::Result<()> {
    if points.len() < 2 {
        return std::result::Result::Ok(());
    }
    let gdi_points: Vec<PointF> = points.iter().map(|&(x, y)| PointF { X: x, Y: y }).collect();
    unsafe {
        let _status = GdipDrawCurve(
            graphics.0,
            pen.0,
            gdi_points.as_ptr(),
            gdi_points.len() as i32,
        );
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipDrawCurve failed: {:?}", _status);
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
        let _status = GdipFillEllipse(graphics.0, brush.0, x, y, width, height);
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipFillEllipse failed: {:?}", _status);
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
        let _status = GdipFillRectangle(graphics.0, brush.0, x, y, width, height);
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipFillRectangle failed: {:?}", _status);
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
        let _status = GdipDrawRectangle(graphics.0, pen.0, x, y, width, height);
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipDrawRectangle failed: {:?}", _status);
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
        let _status = GdipDrawEllipse(graphics.0, pen.0, x, y, width, height);
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipDrawEllipse failed: {:?}", _status);
        }
    }
    std::result::Result::Ok(())
}

pub fn fill_rounded_rectangle(
    graphics: &GraphicsWrapper,
    brush: &BrushWrapper,
    rect: (f32, f32, f32, f32),
    radius: f32,
) -> anyhow::Result<()> {
    unsafe {
        let mut path = std::ptr::null_mut();
        let _ = GdipCreatePath(FillModeAlternate, &mut path);

        let (x, y, w, h) = rect;
        let d = radius * 2.0;

        // Add arcs
        let _ = GdipAddPathArc(path, x, y, d, d, 180.0, 90.0);
        let _ = GdipAddPathArc(path, x + w - d, y, d, d, 270.0, 90.0);
        let _ = GdipAddPathArc(path, x + w - d, y + h - d, d, d, 0.0, 90.0);
        let _ = GdipAddPathArc(path, x, y + h - d, d, d, 90.0, 90.0);

        let _ = GdipClosePathFigure(path);

        let _status = GdipFillPath(graphics.0, brush.0, path);
        let _ = GdipDeletePath(path);
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipFillPath failed: {:?}", _status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_rounded_rectangle(
    graphics: &GraphicsWrapper,
    pen: &PenWrapper,
    rect: (f32, f32, f32, f32),
    radius: f32,
) -> anyhow::Result<()> {
    unsafe {
        let mut path = std::ptr::null_mut();
        let _ = GdipCreatePath(FillModeAlternate, &mut path);

        let (x, y, w, h) = rect;
        let d = radius * 2.0;

        // Add arcs
        let _ = GdipAddPathArc(path, x, y, d, d, 180.0, 90.0);
        let _ = GdipAddPathArc(path, x + w - d, y, d, d, 270.0, 90.0);
        let _ = GdipAddPathArc(path, x + w - d, y + h - d, d, d, 0.0, 90.0);
        let _ = GdipAddPathArc(path, x, y + h - d, d, d, 90.0, 90.0);

        let _ = GdipClosePathFigure(path);

        let _status = GdipDrawPath(graphics.0, pen.0, path);
        let _ = GdipDeletePath(path);
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipDrawPath failed: {:?}", _status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_image_opaque(
    graphics: &GraphicsWrapper,
    bitmap: &super::wrappers::BitmapWrapper,
    dest_rect: (f32, f32, f32, f32),
) -> anyhow::Result<()> {
    unsafe {
        use windows::Win32::Graphics::GdiPlus::*;

        let mut attr = std::ptr::null_mut();
        let _ = GdipCreateImageAttributes(&mut attr);

        #[allow(non_snake_case)]
        let matrix = ColorMatrix {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, // Zero alpha contribution from source
                0.0, 0.0, 0.0, 1.0, 1.0, // Force 1.0 alpha
            ],
        };

        let _ = GdipSetImageAttributesColorMatrix(
            attr,
            ColorAdjustTypeDefault,
            true,
            &matrix,
            std::ptr::null(),
            ColorMatrixFlagsDefault,
        );

        let _status = GdipDrawImageRectRect(
            graphics.0,
            bitmap.0 as *mut GpImage,
            dest_rect.0,
            dest_rect.1,
            dest_rect.2,
            dest_rect.3,
            0.0,
            0.0,
            dest_rect.2,
            dest_rect.3,
            UnitPixel,
            attr,
            0,
            std::ptr::null_mut(),
        );

        let _ = GdipDisposeImageAttributes(attr);

        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipDrawImageRectRect (opaque) failed: {:?}", _status);
        }
    }
    std::result::Result::Ok(())
}

pub fn draw_image_with_attr(
    graphics: &GraphicsWrapper,
    bitmap: &super::wrappers::BitmapWrapper,
    dest_rect: (f32, f32, f32, f32),
    src_rect: (f32, f32, f32, f32),
    attr: *mut windows::Win32::Graphics::GdiPlus::GpImageAttributes,
) -> anyhow::Result<()> {
    unsafe {
        use windows::Win32::Graphics::GdiPlus::*;
        let _status = GdipDrawImageRectRect(
            graphics.0,
            bitmap.0 as *mut GpImage,
            dest_rect.0,
            dest_rect.1,
            dest_rect.2,
            dest_rect.3,
            src_rect.0,
            src_rect.1,
            src_rect.2,
            src_rect.3,
            UnitPixel,
            attr,
            0,
            std::ptr::null_mut(),
        );
        if _status != windows::Win32::Graphics::GdiPlus::Ok {
            anyhow::bail!("GdipDrawImageRectRect with attr failed: {:?}", _status);
        }
    }
    std::result::Result::Ok(())
}
