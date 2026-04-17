use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;

pub trait DrawingToolRenderer: Send + Sync {
    fn render(
        &self,
        hdc: &SafeHDC,
        graphics: Option<&crate::service::win32::gdiplus::GraphicsWrapper>,
        src_hdc: Option<&SafeHDC>,
        cache: &mut crate::service::win32::gdi::GdiCache,
        obj: &DrawingObject,
    ) -> anyhow::Result<()>;

    fn render_gdiplus(
        &self,
        _graphics: &crate::service::win32::gdiplus::GraphicsWrapper,
        _cache: &mut crate::service::win32::gdi::GdiCache,
        _obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn render_gdiplus_with_src(
        &self,
        graphics: &crate::service::win32::gdiplus::GraphicsWrapper,
        _src_hdc: Option<&SafeHDC>,
        cache: &mut crate::service::win32::gdi::GdiCache,
        obj: &DrawingObject,
    ) -> anyhow::Result<()> {
        self.render_gdiplus(graphics, cache, obj)
    }
}
