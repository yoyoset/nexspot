use super::wrappers::{BrushWrapper, GraphicsWrapper};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Graphics::GdiPlus::*;

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
    let text_h = HSTRING::from(text);
    let style = font_style.unwrap_or(FontStyleRegular);

    unsafe {
        let family = create_font_family(font_family)?;

        let mut font = std::ptr::null_mut();
        let _status = GdipCreateFont(family, font_size, style.0, UnitPixel, &mut font);
        if _status != Ok {
            let _ = GdipDeleteFontFamily(family);
            anyhow::bail!("GdipCreateFont failed: {:?}", _status);
        }

        let layout_rect = RectF {
            X: pos.0,
            Y: pos.1,
            Width: 0.0,
            Height: 0.0,
        };

        let _status = GdipDrawString(
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

        if _status != Ok {
            anyhow::bail!("GdipDrawString failed: {:?}", _status);
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
    let text_h = HSTRING::from(text);
    let style = font_style.unwrap_or(FontStyleRegular);

    unsafe {
        let family = create_font_family(font_family)?;

        let mut font = std::ptr::null_mut();
        let _status = GdipCreateFont(family, font_size, style.0, UnitPixel, &mut font);
        if _status != Ok {
            let _ = GdipDeleteFontFamily(family);
            anyhow::bail!("GdipCreateFont failed: {:?}", _status);
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

        let _status = GdipDrawString(
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

        if _status != Ok {
            anyhow::bail!("GdipDrawString failed: {:?}", _status);
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
    let text_h = HSTRING::from(text);
    let style = font_style.unwrap_or(FontStyleRegular);

    unsafe {
        let family = create_font_family(font_family)?;

        let mut font = std::ptr::null_mut();
        let _status = GdipCreateFont(family, font_size, style.0, UnitPixel, &mut font);
        if _status != Ok {
            let _ = GdipDeleteFontFamily(family);
            anyhow::bail!("GdipCreateFont failed: {:?}", _status);
        }

        let layout_rect = RectF {
            X: 0.0,
            Y: 0.0,
            Width: 0.0,
            Height: 0.0,
        };
        let mut bounding_box = RectF::default();

        let _status = GdipMeasureString(
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

        if _status != Ok {
            anyhow::bail!("GdipMeasureString failed: {:?}", _status);
        }
        std::result::Result::Ok(bounding_box)
    }
}

/// Helper to create Font Family with system and private collection fallback
unsafe fn create_font_family(font_family: &str) -> anyhow::Result<*mut GpFontFamily> {
    let mut family = std::ptr::null_mut();
    let font_family_h = HSTRING::from(font_family);

    // 1. Try system font collection
    let mut status = GdipCreateFontFamilyFromName(
        PCWSTR(font_family_h.as_ptr()),
        std::ptr::null_mut(),
        &mut family,
    );

    // 2. Try private font collection if system fails
    if status != Ok {
        let collection = super::get_global_font_collection();
        if !collection.is_null() {
            status = GdipCreateFontFamilyFromName(
                PCWSTR(font_family_h.as_ptr()),
                collection,
                &mut family,
            );

            // 3. Fallback for RemixIcon if name is PascalCase in collection
            if status != Ok && font_family == "remixicon" {
                let alt_name = HSTRING::from("RemixIcon");
                status = GdipCreateFontFamilyFromName(
                    PCWSTR(alt_name.as_ptr()),
                    collection,
                    &mut family,
                );
            }
        }
    }

    if status != Ok {
        anyhow::bail!(
            "GdipCreateFontFamilyFromName failed for font '{}': status {:?}",
            font_family,
            status
        );
    }
    std::result::Result::Ok(family)
}
