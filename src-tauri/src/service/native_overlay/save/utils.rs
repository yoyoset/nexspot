use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// Shared notification for clipboard actions
pub fn notify_copied(app: &AppHandle) {
    use crate::service::l10n;
    let _ = app
        .notification()
        .builder()
        .title(l10n::t(app, "notification.copied_title", "Copied"))
        .body(l10n::t(app, "notification.copied_body", "Image copied to clipboard"))
        .show();
    use tauri::Emitter;
    let _ = app.emit("screenshot-copied", ());
}

/// Helper to save raw RGBA pixels to disk using the image crate, bypassing GDI.
/// This ensures 100% color fidelity for Vello captures.
pub fn save_pixels_to_file(
    app: &AppHandle,
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    template: String,
    folder: Option<String>,
    format: String,
) {
    let app_state = app.state::<crate::app_state::AppState>();
    let captures_dir = if let Some(cp) = folder {
        std::path::PathBuf::from(cp)
    } else {
        let lock = app_state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        lock.resolve_save_path(app)
    };

    if !captures_dir.exists() {
        let _ = std::fs::create_dir_all(&captures_dir);
    }

    let now = chrono::Local::now();
    let mut filename = now.format(&template).to_string();
    if !filename.to_lowercase().ends_with(&format!(".{}", format)) {
        filename.push_str(&format!(".{}", format));
    }
    let file_path = captures_dir.join(filename);

    let quality = app
        .state::<crate::app_state::AppState>()
        .config_state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .config.jpg_quality;

    let extension = format.to_lowercase();
    let result = if extension == "jpg" || extension == "jpeg" {
        let mut rgb = Vec::with_capacity((width * height * 3) as usize);
        for c in pixels.chunks_exact(4) {
            rgb.extend_from_slice(&[c[0], c[1], c[2]]);
        }
        std::fs::File::create(&file_path).map_err(|e| image::ImageError::IoError(e)).and_then(|file| {
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
            if let Some(buf) = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, rgb) {
                encoder.encode_image(&buf).map_err(|e| image::ImageError::Encoding(image::error::EncodingError::new(image::error::ImageFormatHint::Exact(image::ImageFormat::Jpeg), e)))
            } else {
                 Err(image::ImageError::Parameter(image::error::ParameterError::from_kind(image::error::ParameterErrorKind::DimensionMismatch)))
            }
        })
    } else {
        image::save_buffer(&file_path, &pixels, width, height, image::ColorType::Rgba8)
    };

    if let Err(e) = result {
        log::error!("Failed to save image to {:?}: {}", file_path, e);
    } else {
        crate::service::activity::log_activity(
            app,
            "screenshot",
            Some(file_path.display().to_string()),
            None,
        );
        use crate::service::l10n;
        let _ = app
            .notification()
            .builder()
            .title(l10n::t(app, "backend.notification.saved_title", "Saved"))
            .body(format!(
                "{}: {}",
                l10n::t(app, "backend.notification.saved_body", "Image saved locally"),
                file_path.display()
            ))
            .show();
        use tauri::Emitter;
        let _ = app.emit("screenshot-saved", ());
    }
}
