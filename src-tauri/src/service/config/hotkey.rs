use super::error::ConfigError;
use super::types::{AIShortcut, AppConfig, HotkeyAction};
use global_hotkey::hotkey::HotKey;
use global_hotkey::GlobalHotKeyManager;
use std::collections::HashMap;

/// Standardizes shortcut strings for global-hotkey parsing.
/// Frontend: "Control+Alt+S" -> Backend: "control+alt+keys"
fn normalize_hotkey(s: &str) -> String {
    let parts: Vec<String> = s
        .split('+')
        .map(|p| p.trim().to_lowercase())
        .map(|p| match p.as_str() {
            "ctrl" | "control" => "control".to_string(),
            "win" | "meta" | "super" => "super".to_string(),
            // Function keys: f1, f2...
            f if f.starts_with('f') && f.len() > 1 && f[1..].chars().all(|c| c.is_numeric()) => {
                f.to_string()
            }
            // Numeric keys: global-hotkey expects digit0...
            d if d.len() == 1 && d.chars().next().unwrap().is_numeric() => format!("digit{}", d),
            // Alphanumeric keys: global-hotkey expects keya...
            a if a.len() == 1 && a.chars().next().unwrap().is_alphabetic() => format!("key{}", a),
            // Special keys
            "space" => "space".to_string(),
            "enter" | "return" => "return".to_string(),
            "escape" | "esc" => "escape".to_string(),
            "backspace" => "backspace".to_string(),
            "delete" | "del" => "delete".to_string(),
            "insert" | "ins" => "insert".to_string(),
            "home" => "home".to_string(),
            "end" => "end".to_string(),
            "pageup" | "pgup" => "pageup".to_string(),
            "pagedown" | "pgdn" => "pagedown".to_string(),
            "tab" => "tab".to_string(),
            "up" => "up".to_string(),
            "down" => "down".to_string(),
            "left" => "left".to_string(),
            "right" => "right".to_string(),
            "printscreen" | "prtsc" => "printscreen".to_string(),
            "scrolllock" => "scrolllock".to_string(),
            "pause" => "pause".to_string(),
            other => other.to_string(),
        })
        .collect();
    parts.join("+")
}

fn check_global_conflict(config: &AppConfig, normalized: &str, exclude_id: &str) -> Option<String> {
    if normalized.is_empty() {
        return None;
    }
    // Check workflows
    if let Some(w) = config
        .workflows
        .iter()
        .find(|w| w.id != exclude_id && normalize_hotkey(&w.shortcut) == normalized)
    {
        return Some(w.label.clone());
    }
    // Check AI shortcuts
    if let Some(s) = config.ai_shortcuts.iter().find(|s| {
        s.id != exclude_id
            && s.shortcut.as_deref().map(|sk| normalize_hotkey(sk)) == Some(normalized.to_string())
    }) {
        return Some(s.name.clone());
    }
    None
}

pub fn update_shortcut(
    manager: &GlobalHotKeyManager,
    config: &mut AppConfig,
    last_errors: &mut Vec<String>,
    id: &str,
    new_keys: &str,
) -> Result<bool, ConfigError> {
    let new_keys = new_keys.trim();
    if new_keys.is_empty() {
        return Err(ConfigError::Empty);
    }

    let normalized = normalize_hotkey(new_keys);
    let new_hotkey = normalized
        .parse::<HotKey>()
        .map_err(|_| ConfigError::InvalidFormat)?;

    // Conflict check (Global)
    if let Some(conflict_name) = check_global_conflict(config, &normalized, id) {
        return Err(ConfigError::Conflict(conflict_name));
    }

    // Find the workflow to update
    let w_idx = config
        .workflows
        .iter()
        .position(|w| w.id == id)
        .ok_or(ConfigError::NotFound)?;

    let old_keys = config.workflows[w_idx].shortcut.clone();

    if old_keys == new_keys {
        return Ok(false);
    }

    // 1. Unregister OLD
    if let Ok(old_hotkey) = normalize_hotkey(&old_keys).parse::<HotKey>() {
        let _ = manager.unregister(old_hotkey);
    }

    // 2. Register NEW
    match manager.register(new_hotkey) {
        Ok(_) => {
            log::info!("Registered Workflow {} -> {}", id, new_keys);
            config.workflows[w_idx].shortcut = new_keys.to_string();

            // Clear from error tracking
            last_errors.retain(|e| !e.contains(new_keys));
            Ok(true)
        }
        Err(e) => {
            log::error!("Failed to register {} ({}): {:?}", id, new_keys, e);
            // Attempt to re-register OLD (FIX: use normalization)
            if let Ok(old_hotkey) = normalize_hotkey(&old_keys).parse::<HotKey>() {
                let _ = manager.register(old_hotkey);
            }
            // Add identifying info to error for UI red-status
            Err(ConfigError::RegistrationFailed(format!(
                "{}:{}",
                id, new_keys
            )))
        }
    }
}

pub fn add_ai_shortcut(
    _manager: &GlobalHotKeyManager,
    config: &mut AppConfig,
    shortcut: AIShortcut,
) -> Result<bool, ConfigError> {
    // 1. Check ID uniqueness
    if config.ai_shortcuts.iter().any(|s| s.id == shortcut.id) {
        return Err(ConfigError::Conflict(format!(
            "ID {} already exists",
            shortcut.id
        )));
    }

    config.ai_shortcuts.push(shortcut);
    Ok(true)
}

pub fn add_workflow(
    manager: &GlobalHotKeyManager,
    config: &mut AppConfig,
    workflow: crate::service::config::types::CaptureWorkflow,
) -> Result<bool, ConfigError> {
    // 1. Check ID uniqueness
    if config.workflows.iter().any(|w| w.id == workflow.id) {
        return Err(ConfigError::Conflict(format!(
            "ID {} already exists",
            workflow.id
        )));
    }

    // Conflict check (Global)
    let normalized_wf = normalize_hotkey(&workflow.shortcut);
    if let Some(conflict_name) = check_global_conflict(config, &normalized_wf, &workflow.id) {
        return Err(ConfigError::Conflict(conflict_name));
    }

    // 4. Register
    if workflow.enabled {
        if let Ok(hotkey) = normalize_hotkey(&workflow.shortcut).parse::<HotKey>() {
            manager.register(hotkey).map_err(|_| {
                ConfigError::RegistrationFailed(format!("{}:{}", workflow.id, workflow.shortcut))
            })?;
        }
    }

    config.workflows.push(workflow);
    Ok(true)
}

pub fn remove_workflow(
    manager: &GlobalHotKeyManager,
    config: &mut AppConfig,
    id: &str,
) -> Result<bool, ConfigError> {
    let idx = config
        .workflows
        .iter()
        .position(|w| w.id == id)
        .ok_or(ConfigError::NotFound)?;

    // System workflows cannot be removed
    if config.workflows[idx].is_system {
        return Err(ConfigError::Other(
            "Cannot delete system workflow".to_string(),
        ));
    }

    let workflow = config.workflows.remove(idx);

    // Unregister
    if let Ok(hotkey) = normalize_hotkey(&workflow.shortcut).parse::<HotKey>() {
        let _ = manager.unregister(hotkey);
    }

    Ok(true)
}

pub fn remove_ai_shortcut(
    _manager: &GlobalHotKeyManager,
    config: &mut AppConfig,
    id: &str,
) -> Result<bool, ConfigError> {
    let idx = config
        .ai_shortcuts
        .iter()
        .position(|s| s.id == id)
        .ok_or(ConfigError::NotFound)?;
    config.ai_shortcuts.remove(idx);
    Ok(true)
}

pub fn update_ai_shortcut(
    _manager: &GlobalHotKeyManager,
    config: &mut AppConfig,
    id: &str,
    mut new_shortcut: AIShortcut,
) -> Result<bool, ConfigError> {
    let idx = config
        .ai_shortcuts
        .iter()
        .position(|s| s.id == id)
        .ok_or(ConfigError::NotFound)?;

    new_shortcut.id = id.to_string();
    config.ai_shortcuts[idx] = new_shortcut;
    Ok(true)
}

pub fn register_all(
    manager: &GlobalHotKeyManager,
    config: &AppConfig,
    errors: &mut Vec<String>,
) -> HashMap<u32, HotkeyAction> {
    errors.clear();
    let mut map = HashMap::new();

    // Register Workflows
    for w in &config.workflows {
        // NOTE: We try to register even if disabled in memory,
        // to check for external occupation, but user wants 'Red/Green'
        // and simplified 'always enabled' logic.
        if w.enabled {
            if let Ok(hotkey) = normalize_hotkey(&w.shortcut).parse::<HotKey>() {
                match manager.register(hotkey) {
                    Ok(_) => {
                        map.insert(hotkey.id(), HotkeyAction::Workflow(w.clone()));
                    }
                    Err(global_hotkey::Error::AlreadyRegistered(_)) => {
                        map.insert(hotkey.id(), HotkeyAction::Workflow(w.clone()));
                    }
                    Err(e) => {
                        log::error!("Init error: {} ({}): {:?}", w.label, w.shortcut, e);
                        // Store the ID:Label pair for precise UI feedback
                        errors.push(format!("{}:{}", w.id, w.label));
                    }
                }
            }
        }
    }
    // AI Shortcuts are now macros in the toolbar, NOT global hotkeys.
    // They don't need to be registered in the system-wide hotkey manager.
    map
}

pub fn unregister_all(manager: &GlobalHotKeyManager, config: &AppConfig) {
    for w in &config.workflows {
        if let Ok(hotkey) = normalize_hotkey(&w.shortcut).parse::<HotKey>() {
            let _ = manager.unregister(hotkey);
        }
    }
    for _ in &config.ai_shortcuts {
        // AI Shortcuts are no longer registered as hotkeys
    }
}
