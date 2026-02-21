use std::fs;
use std::path::Path;

pub fn register_custom_fonts(fonts_dir: &Path) -> anyhow::Result<()> {
    if let Ok(entries) = fs::read_dir(fonts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "ttf" || ext_str == "otf" {
                    log::info!("Registering custom font: {:?}", path);
                    let _ = crate::service::win32::gdi::register_font(&path);
                }
            }
        }
    }
    Ok(())
}
