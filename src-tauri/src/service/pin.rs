use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum PinData {
    Text(String),
    ImageBase64(String), // base64 encoded image
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PinItem {
    pub id: String,
    pub data: PinData,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct PinState {
    pub pins: Arc<Mutex<Vec<PinItem>>>,
}

impl PinState {
    pub fn new() -> Self {
        Self {
            pins: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_pin(&self, id: String, data: PinData) {
        let mut pins = self.pins.lock().unwrap_or_else(|e| e.into_inner());
        // Remove existing if any
        pins.retain(|p| p.id != id);
        pins.push(PinItem {
            id,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });
        // Optionally sort by timestamp if we want newest first, or keep appended
    }

    pub fn get_pin(&self, id: &str) -> Option<PinItem> {
        let pins = self.pins.lock().unwrap_or_else(|e| e.into_inner());
        pins.iter().find(|p| p.id == id).cloned()
    }

    pub fn get_all_pins(&self) -> Vec<PinItem> {
        let pins = self.pins.lock().unwrap_or_else(|e| e.into_inner());
        pins.clone()
    }

    pub fn remove_pin(&self, id: &str) {
        let mut pins = self.pins.lock().unwrap_or_else(|e| e.into_inner());
        pins.retain(|p| p.id != id);
    }

    pub fn clear_all(&self) {
        let mut pins = self.pins.lock().unwrap_or_else(|e| e.into_inner());
        pins.clear();
    }
}

pub fn open_pin_collection_window(app: &AppHandle) -> tauri::Result<()> {
    let window_label = "pin-collection";

    if let Some(window) = app.get_webview_window(window_label) {
        // Just bring to front if already exists
        let _ = window.show().map_err(|e| log::warn!("Show error: {}", e));
        let _ = window
            .set_focus()
            .map_err(|e| log::warn!("Focus error: {}", e));
        return Ok(());
    }

    // URL with Hash for routing
    let url = WebviewUrl::App("index.html#/pin-collection".into());

    let win_builder = WebviewWindowBuilder::new(app, window_label, url)
        .title("NexSpot Collection")
        .inner_size(640.0, 600.0)
        .min_inner_size(320.0, 400.0)
        .transparent(true)
        .decorations(false)
        .always_on_top(false)
        .skip_taskbar(false)
        .resizable(true);

    let window = win_builder.build()?;

    // Let it appear at a reasonable size centered
    let _ = window.center();

    Ok(())
}
