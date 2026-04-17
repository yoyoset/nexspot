use std::sync::Once;
use windows::Win32::Graphics::GdiPlus::*;

mod drawing;
mod text;
mod wrappers;

// Re-export public items for convenience and backward compatibility
pub use self::drawing::*;
pub use self::text::*;
pub use self::wrappers::*;

static GDI_PLUS_INIT: Once = Once::new();
static mut GDI_PLUS_TOKEN: usize = 0;
static mut PRIVATE_FONT_COLLECTION: Option<*mut GpFontCollection> = None;

pub fn init_gdiplus() {
    GDI_PLUS_INIT.call_once(|| {
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            ..Default::default()
        };
        let mut token: usize = 0;
        let mut output = GdiplusStartupOutput::default();
        unsafe {
            let _ = GdiplusStartup(&mut token, &input, &mut output as *mut _);
            GDI_PLUS_TOKEN = token;

            let mut collection = std::ptr::null_mut();
            if GdipNewPrivateFontCollection(&mut collection) == Ok {
                PRIVATE_FONT_COLLECTION = Some(collection);
            }
        }
    });
}

pub fn register_font_for_gdiplus(data: &[u8]) {
    init_gdiplus(); // Ensure initialized
    unsafe {
        if let Some(collection) = PRIVATE_FONT_COLLECTION {
            let status =
                GdipPrivateAddMemoryFont(collection, data.as_ptr() as *const _, data.len() as i32);
            if status == Ok {
                log::info!("GDI+ Private font added successfully");
                list_private_font_families();
            } else {
                log::error!("GDI+ GdipPrivateAddMemoryFont failed: {:?}", status);
            }
        }
    }
}

pub fn list_private_font_families() {
    unsafe {
        if let Some(collection) = PRIVATE_FONT_COLLECTION {
            let mut count = 0;
            if GdipGetFontCollectionFamilyCount(collection, &mut count) == Ok {
                log::info!("GDI+ PrivateFontCollection has {} families", count);
                if count > 0 {
                    let mut families = vec![std::ptr::null_mut(); count as usize];
                    let mut found = 0;
                    if GdipGetFontCollectionFamilyList(collection, &mut families, &mut found) == Ok
                    {
                        for i in 0..found {
                            let mut name = [0u16; 32];
                            if GdipGetFamilyName(families[i as usize], &mut name, 0) == Ok {
                                let name_str = String::from_utf16_lossy(&name);
                                log::info!(
                                    "GDI+ Private Family [{}]: {}",
                                    i,
                                    name_str.trim_matches('\0')
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn get_global_font_collection() -> *mut GpFontCollection {
    unsafe { PRIVATE_FONT_COLLECTION.unwrap_or(std::ptr::null_mut()) }
}
