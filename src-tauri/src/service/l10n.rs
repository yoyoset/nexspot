use crate::AppState;
use tauri::Manager;
use std::collections::HashMap;
use std::sync::OnceLock;

// 单一事实来源：直接从前端 locale 目录编译时包含
static ZH_DICT_STR: &str = include_str!("../../../src/locales/zh.json");
static EN_DICT_STR: &str = include_str!("../../../src/locales/en.json");

static DICTIONARY: OnceLock<(HashMap<String, String>, HashMap<String, String>)> = OnceLock::new();

fn get_dicts() -> &'static (HashMap<String, String>, HashMap<String, String>) {
    DICTIONARY.get_or_init(|| {
        let mut zh_map = HashMap::new();
        let mut en_map = HashMap::new();

        fn flatten_json(value: &serde_json::Value, prefix: String, map: &mut HashMap<String, String>) {
            match value {
                serde_json::Value::Object(obj) => {
                    for (k, v) in obj {
                        let new_prefix = if prefix.is_empty() {
                            k.clone()
                        } else {
                            format!("{}.{}", prefix, k)
                        };
                        flatten_json(v, new_prefix, map);
                    }
                }
                serde_json::Value::String(s) => {
                    map.insert(prefix, s.clone());
                }
                _ => {}
            }
        }

        if let Ok(zh_json) = serde_json::from_str::<serde_json::Value>(ZH_DICT_STR) {
            flatten_json(&zh_json, "".to_string(), &mut zh_map);
        }
        if let Ok(en_json) = serde_json::from_str::<serde_json::Value>(EN_DICT_STR) {
            flatten_json(&en_json, "".to_string(), &mut en_map);
        }

        (zh_map, en_map)
    })
}

pub fn t<R: tauri::Runtime>(app: &tauri::AppHandle<R>, key_path: &str, default_fallback: &str) -> String {
    let lang = app
        .try_state::<AppState>()
        .and_then(|s| {
            s.config_state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .config
                .language
                .clone()
                .into()
        })
        .unwrap_or_else(|| "zh".to_string());

    let (zh_dict, en_dict) = get_dicts();
    
    let is_zh = lang == "zh" || lang == "zh-CN";
    let target_dict = if is_zh { zh_dict } else { en_dict };

    // 优先尝试 backend 路径，再尝试原始路径
    let backend_key = format!("backend.{}", key_path);
    if let Some(val) = target_dict.get(&backend_key) {
        return val.clone();
    }

    target_dict.get(key_path).cloned().unwrap_or_else(|| default_fallback.to_string())
}

pub fn t_with_args<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    key_path: &str,
    args: &[(&str, &str)],
) -> String {
    let mut text = t(app, key_path, key_path);
    for (key, val) in args {
        text = text.replace(&format!("{{{}}}", key), val);
    }
    text
}

