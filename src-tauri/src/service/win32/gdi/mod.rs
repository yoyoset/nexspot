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
