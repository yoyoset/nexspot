use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub id: String,
    pub timestamp: u64,
    pub activity_type: String, // "screenshot", "ocr", "scrolling"
    pub file_path: Option<String>,
    pub metadata: Option<String>, // e.g. OCR text snippet
}

pub struct ActivityState {
    pub entries: Arc<Mutex<Vec<ActivityEntry>>>,
    pub storage_path: PathBuf,
}

impl ActivityState {
    pub fn new(app_dir: PathBuf) -> Self {
        let storage_path = app_dir.join("activity.json");
        
        // Ensure directory exists
        if let Some(parent) = storage_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let entries = if let Ok(data) = fs::read_to_string(&storage_path) {
            serde_json::from_str(&data).unwrap_or_else(|e| {
                log::error!("Failed to parse activity.json: {:?}", e);
                Vec::new()
            })
        } else {
            Vec::new()
        };
        
        Self {
            entries: Arc::new(Mutex::new(entries)),
            storage_path,
        }
    }

    pub fn add_entry(&self, entry: ActivityEntry) {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries.insert(0, entry);
        
        // Pruning: Max 100 entries for efficiency
        if entries.len() > 100 {
            entries.truncate(100);
        }
        
        if let Err(e) = self.save(&entries) {
            log::error!("Failed to save activity: {:?}", e);
        }
    }

    pub fn get_all(&self) -> Vec<ActivityEntry> {
        self.entries.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn save(&self, entries: &Vec<ActivityEntry>) -> anyhow::Result<()> {
        let data = serde_json::to_string_pretty(entries)?;
        fs::write(&self.storage_path, data)?;
        Ok(())
    }
}

// Tauri Command to fetch activity
#[tauri::command]
pub fn get_activity(app_handle: AppHandle) -> Vec<ActivityEntry> {
    let state = app_handle.state::<crate::app_state::AppState>();
    state.activity_state.get_all()
}

// Utility to add entry from anywhere
pub fn log_activity(app_handle: &AppHandle, activity_type: &str, file_path: Option<String>, metadata: Option<String>) {
    use uuid::Uuid;
    let entry = ActivityEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        activity_type: activity_type.to_string(),
        file_path,
        metadata,
    };

    if let Some(state) = app_handle.try_state::<crate::app_state::AppState>() {
        state.activity_state.add_entry(entry);
        let _ = app_handle.emit("activity://updated", ());
    }
}
