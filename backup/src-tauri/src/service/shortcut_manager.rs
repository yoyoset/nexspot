use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Shortcut {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub color: String,
    pub enabled: bool,
}

pub fn get_shortcuts_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.push("shortcuts.json");
    path
}

pub fn load_shortcuts() -> Vec<Shortcut> {
    let path = get_shortcuts_path();
    if !path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_shortcuts(shortcuts: Vec<Shortcut>) -> Result<(), String> {
    let path = get_shortcuts_path();
    let content = serde_json::to_string_pretty(&shortcuts).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}
