pub mod engines;
pub mod traits;

use engines::windows::WindowsNativeOcr;
use traits::OcrEngine;

pub fn get_engine(id: &str) -> Box<dyn OcrEngine> {
    match id {
        "windows" | "default" | "" => Box::new(WindowsNativeOcr),
        _ => Box::new(WindowsNativeOcr), // Fallback
    }
}
