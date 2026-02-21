use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone)]
pub struct PinState {
    pub pins: Arc<Mutex<HashMap<String, String>>>, // ID -> Content (Markdown)
}

impl PinState {
    pub fn new() -> Self {
        Self {
            pins: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_pin(&self, id: String, content: String) {
        let mut pins = self.pins.lock().unwrap();
        pins.insert(id, content);
    }

    pub fn get_content(&self, id: &str) -> Option<String> {
        let pins = self.pins.lock().unwrap();
        pins.get(id).cloned()
    }

    pub fn remove_pin(&self, id: &str) {
        let mut pins = self.pins.lock().unwrap();
        pins.remove(id);
    }
}

pub fn create_text_pin_window(app: &AppHandle, id: &str) -> tauri::Result<()> {
    // Window ID must be unique and valid
    let window_label = format!("pin-{}", id);

    // URL with Hash for routing
    let url = WebviewUrl::App(format!("index.html#/text-pin?id={}", id).into());

    let win_builder = WebviewWindowBuilder::new(app, &window_label, url)
        .title("Text Pin")
        .inner_size(400.0, 300.0)
        .min_inner_size(200.0, 150.0)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(false) // Pins should probably be in taskbar? Or not? "Sticky Note" style usually is.
        .resizable(true);

    let window = win_builder.build()?;

    // Position? Maybe center or cascaded?
    // For now, let OS decide or center.
    let _ = window.center();

    Ok(())
}
