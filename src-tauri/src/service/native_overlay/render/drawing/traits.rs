use crate::service::native_overlay::state::DrawingObject;
use crate::service::win32::gdi::SafeHDC;

pub trait DrawingToolRenderer: Send + Sync {
    fn render(&self, hdc: &SafeHDC, obj: &DrawingObject) -> anyhow::Result<()>;
}
