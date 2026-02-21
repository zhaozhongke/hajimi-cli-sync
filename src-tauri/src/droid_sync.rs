use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::utils;

const DROID_DIR: &str = ".factory";
const DROID_CONFIG_FILE: &str = "settings.json";
use crate::utils::BACKUP_SUFFIX;
const AG_ID_PREFIX: &str = "custom:AG-";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DroidStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub is_synced: bool,
    pub has_backup: bool,
    pub current_base_url: Option<String>,
    pub files: Vec<String>,
    pub synced_count: usize,
}

fn get_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(DROID_DIR).join(DROID_CONFIG_FILE))
}

pub fn check_droid_installed() -> (bool, Option<String>) {
    match utils::resolve_executable("droid") {
        Some(path) => {
            let version = utils::get_cli_version(&path);
            (true, version.or_else(|| Some("unknown".to_string())))
        }
        None => (false, None),
    }
}

fn count_synced_models(json: &Value) -> (usize, Option<String>) {
    let mut count = 0;
    let mut first_url = None;

    if let Some(arr) = json.get("customModels").and_then(|v| v.as_array()) {
        for m in arr {
            let id = m.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            if !id.starts_with(AG_ID_PREFIX) {
                continue;
            }
            count += 1;
            if first_url.is_none() {
                first_url = m
                    .get("baseUrl")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }
    }
    (count, first_url)
}

pub fn get_sync_status(proxy_url: &str) -> (bool, bool, Option<String>, usize) {
    let config_path = match get_config_path() {
        Some(p) => p,
        None => return (false, false, None, 0),
    };

    let backup_path = config_path.with_file_name(format!("{DROID_CONFIG_FILE}{BACKUP_SUFFIX}"));
    let has_backup = backup_path.exists();

    if !config_path.exists() {
        return (false, has_backup, None, 0);
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return (false, has_backup, None, 0),
    };

    let json: Value = serde_json::from_str(&content).unwrap_or_default();
    let (synced_count, first_url) = count_synced_models(&json);
    // is_synced: must have AG models AND their baseUrl must match current proxy
    let is_synced = synced_count > 0
        && first_url
            .as_deref()
            .is_some_and(|u| utils::urls_match(u, proxy_url));
    (is_synced, has_backup, first_url, synced_count)
}

fn build_droid_custom_models(proxy_url: &str, api_key: &str, model_ids: &[&str]) -> Vec<Value> {
    model_ids
        .iter()
        .map(|model_id| {
            serde_json::json!({
                "id": format!("{}{}", AG_ID_PREFIX, model_id),
                "name": format!("[Hajimi] {}", model_id),
                "baseUrl": proxy_url,
                "apiKey": api_key,
                "provider": "anthropic"
            })
        })
        .collect()
}

pub fn sync_droid_config(
    proxy_url: &str,
    api_key: &str,
    model: Option<&str>,
) -> Result<usize, String> {
    let config_path = get_config_path()
        .ok_or_else(|| "Failed to get Droid config directory (home dir not found)".to_string())?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {parent:?}: {e}"))?;
    }

    utils::create_rotated_backup(&config_path, BACKUP_SUFFIX)?;

    let mut config: Value = if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {e}"))?;
        serde_json::from_str(&content).unwrap_or_else(|e| {
            tracing::warn!("[droid_sync] Config corrupted, starting fresh: {}", e);
            serde_json::json!({})
        })
    } else {
        serde_json::json!({})
    };

    if !config.is_object() {
        config = serde_json::json!({});
    }

    let default_models: Vec<&str> = vec![
        "claude-sonnet-4-5",
        "claude-sonnet-4-5-thinking",
        "claude-opus-4-5-thinking",
        "gemini-3-pro-high",
        "gemini-3-pro-low",
        "gemini-3-flash",
        "gemini-2.5-flash",
        "gemini-2.5-pro",
        "gpt-4o",
        "o3",
    ];

    let models_to_sync: Vec<&str> = if let Some(m) = model {
        vec![m]
    } else {
        default_models
    };

    let new_ag_models = build_droid_custom_models(proxy_url, api_key, &models_to_sync);
    let ag_count = new_ag_models.len();

    // Preserve user's non-AG custom models
    let mut existing_non_ag: Vec<Value> = Vec::new();
    if let Some(arr) = config.get("customModels").and_then(|v| v.as_array()) {
        for m in arr {
            let id = m.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            if !id.starts_with(AG_ID_PREFIX) {
                existing_non_ag.push(m.clone());
            }
        }
    }

    let mut merged = existing_non_ag;
    merged.extend(new_ag_models);

    let obj = config
        .as_object_mut()
        .ok_or_else(|| "Internal error: config is not an object".to_string())?;
    obj.insert("customModels".to_string(), Value::Array(merged));

    let content = utils::to_json_pretty(&config)?;
    utils::atomic_write(&config_path, &content)?;

    Ok(ag_count)
}

pub fn restore_droid_config() -> Result<(), String> {
    let config_path =
        get_config_path().ok_or_else(|| "Failed to get Droid config directory".to_string())?;

    let backup_path = config_path.with_file_name(format!("{DROID_CONFIG_FILE}{BACKUP_SUFFIX}"));
    if backup_path.exists() {
        fs::rename(&backup_path, &config_path)
            .map_err(|e| format!("Failed to restore config: {e}"))?;
        Ok(())
    } else {
        Err("No backup file found".to_string())
    }
}

pub fn read_droid_config_content() -> Result<String, String> {
    let config_path =
        get_config_path().ok_or_else(|| "Failed to get Droid config directory".to_string())?;

    if !config_path.exists() {
        return Ok("{}".to_string());
    }

    fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {e}"))
}

pub fn write_droid_config_content(content: &str) -> Result<(), String> {
    let config_path = get_config_path().ok_or_else(|| "Config path not found".to_string())?;
    serde_json::from_str::<serde_json::Value>(content)
        .map_err(|e| format!("Invalid JSON: {e}"))?;
    utils::atomic_write(&config_path, content).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_synced_models_empty() {
        let json = serde_json::json!({});
        let (count, url) = count_synced_models(&json);
        assert_eq!(count, 0);
        assert!(url.is_none());
    }

    #[test]
    fn test_count_synced_models_with_ag_models() {
        let json = serde_json::json!({
            "customModels": [
                { "id": "custom:AG-claude-sonnet-4-5", "baseUrl": "https://example.com" },
                { "id": "my-custom-model", "baseUrl": "https://other.com" },
                { "id": "custom:AG-gpt-4o", "baseUrl": "https://example.com" }
            ]
        });
        let (count, url) = count_synced_models(&json);
        assert_eq!(count, 2);
        assert_eq!(url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_build_droid_custom_models() {
        let models = build_droid_custom_models("https://example.com", "sk-test", &["gpt-4o"]);
        assert_eq!(models.len(), 1);
        assert_eq!(models[0]["id"], "custom:AG-gpt-4o");
        assert_eq!(models[0]["baseUrl"], "https://example.com");
        assert_eq!(models[0]["apiKey"], "sk-test");
    }
}
