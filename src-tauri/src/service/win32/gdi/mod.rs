pub mod cache;
pub mod dc;
pub mod effects;
pub mod resources;
pub mod shapes;
pub mod text;

// Re-export common foundation types used in GDI
pub use windows::Win32::Foundation::{COLORREF, RECT};
pub use windows::Win32::Graphics::Gdi::{BACKGROUND_MODE, ROP_CODE, SRCCOPY};

// Re-export everything for backward compatibility
pub use cache::*;
pub use dc::*;
pub use effects::*;
pub use resources::*;
pub use shapes::*;
pub use text::*;

/// Converts standard ARGB (0xAARRGGBB) to Windows GDI COLORREF (0x00BBGGRR).
/// Note: GDI ignores Alpha, so 0x00 is used for the high byte.
#[inline]
pub fn to_colorref(argb: u32) -> COLORREF {
    let r = (argb >> 16) & 0xFF;
    let g = (argb >> 8) & 0xFF;
    let b = argb & 0xFF;
    COLORREF(r | (g << 8) | (b << 16))
}
