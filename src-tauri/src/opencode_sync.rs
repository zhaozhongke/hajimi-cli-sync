use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::utils;

const OPENCODE_CONFIG_FILE: &str = "opencode.json";
use crate::utils::BACKUP_SUFFIX;
const PROVIDER_ID: &str = "hajimi";

/// Normalize base URL to ensure it ends with `/v1`
fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{}/v1", trimmed)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpencodeStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub is_synced: bool,
    pub has_backup: bool,
    pub current_base_url: Option<String>,
    pub files: Vec<String>,
}

fn get_opencode_dir() -> Option<PathBuf> {
    // Respect XDG_CONFIG_HOME on Linux (consistent with lib.rs::get_config_folder_path).
    let config_base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".config"))
                .unwrap_or_else(|| PathBuf::from(".config"))
        });
    Some(config_base.join("opencode"))
}

fn get_config_path() -> Option<PathBuf> {
    get_opencode_dir().map(|dir| dir.join(OPENCODE_CONFIG_FILE))
}

pub fn check_opencode_installed() -> (bool, Option<String>) {
    match utils::resolve_executable("opencode") {
        Some(path) => {
            let version = utils::get_cli_version(&path);
            (true, version.or_else(|| Some("unknown".to_string())))
        }
        None => (false, None),
    }
}

pub fn get_sync_status(proxy_url: &str) -> (bool, bool, Option<String>) {
    let config_path = match get_config_path() {
        Some(p) => p,
        None => return (false, false, None),
    };

    let backup_path =
        config_path.with_file_name(format!("{}{}", OPENCODE_CONFIG_FILE, BACKUP_SUFFIX));
    let has_backup = backup_path.exists();

    if !config_path.exists() {
        return (false, has_backup, None);
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return (false, has_backup, None),
    };

    let json: Value = serde_json::from_str(&content).unwrap_or_default();
    let normalized_proxy = normalize_base_url(proxy_url);

    let ag_url = json
        .get("provider")
        .and_then(|p| p.get(PROVIDER_ID))
        .and_then(|prov| prov.get("options"))
        .and_then(|o| o.get("baseURL"))
        .and_then(|v| v.as_str());
    let ag_key = json
        .get("provider")
        .and_then(|p| p.get(PROVIDER_ID))
        .and_then(|prov| prov.get("options"))
        .and_then(|o| o.get("apiKey"))
        .and_then(|v| v.as_str());

    let mut is_synced = true;
    let mut current_base_url = None;

    if let (Some(url), Some(_key)) = (ag_url, ag_key) {
        current_base_url = Some(url.to_string());
        if normalize_base_url(url) != normalized_proxy {
            is_synced = false;
        }
    } else {
        is_synced = false;
    }

    (is_synced, has_backup, current_base_url)
}

/// Fetch model IDs from the proxy's /v1/models endpoint.
/// Returns a map of model_id -> { "name": model_id } for opencode's models format.
/// On any failure, returns an empty map (sync still proceeds without models).
async fn fetch_models_from_proxy(base_url: &str, api_key: &str) -> serde_json::Map<String, Value> {
    let models_url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return serde_json::Map::new(),
    };

    let resp = match client
        .get(&models_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return serde_json::Map::new(),
    };

    let body: Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return serde_json::Map::new(),
    };

    let mut models = serde_json::Map::new();
    if let Some(data) = body.get("data").and_then(|v| v.as_array()) {
        for item in data {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                let is_reasoning = id.contains("thinking") || id.contains("pro");
                let is_claude = id.contains("claude");
                let is_gemini = id.contains("gemini");
                let is_image = id.contains("image");
                let supports_attachment = is_claude || is_gemini;

                let mut model_obj = serde_json::json!({ "name": id });
                if let Some(obj) = model_obj.as_object_mut() {
                    if supports_attachment {
                        obj.insert("attachment".to_string(), Value::Bool(true));
                    }
                    if is_reasoning {
                        obj.insert("reasoning".to_string(), Value::Bool(true));
                    }
                    if !is_image {
                        obj.insert("tool_call".to_string(), Value::Bool(true));
                    }
                }
                models.insert(id.to_string(), model_obj);
            }
        }
    }
    models
}

pub async fn sync_opencode_config(proxy_url: &str, api_key: &str) -> Result<(), String> {
    let config_path = get_config_path().ok_or_else(|| {
        "Failed to get OpenCode config directory (home dir not found)".to_string()
    })?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
    }

    utils::create_rotated_backup(&config_path, BACKUP_SUFFIX).map_err(|e| e.to_string())?;

    let mut config: Value = if config_path.exists() {
        fs::read_to_string(&config_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !config.is_object() {
        config = serde_json::json!({});
    }

    if config.get("$schema").is_none() {
        config["$schema"] = Value::String("https://opencode.ai/config.json".to_string());
    }

    let normalized_url = normalize_base_url(proxy_url);

    // Fetch models from proxy before any mutable borrows
    let fetched_models = fetch_models_from_proxy(&normalized_url, api_key).await;

    // Ensure provider object exists
    if !config.get("provider").map_or(false, |v| v.is_object()) {
        config["provider"] = serde_json::json!({});
    }

    if let Some(provider) = config.get_mut("provider").and_then(|p| p.as_object_mut()) {
        if !provider.get(PROVIDER_ID).map_or(false, |v| v.is_object()) {
            provider.insert(PROVIDER_ID.to_string(), serde_json::json!({}));
        }
        if let Some(ag_provider) = provider.get_mut(PROVIDER_ID) {
            if let Some(obj) = ag_provider.as_object_mut() {
                obj.insert(
                    "npm".to_string(),
                    Value::String("@ai-sdk/openai".to_string()),
                );
                obj.insert("name".to_string(), Value::String("Hajimi".to_string()));
            }

            if !ag_provider.get("options").map_or(false, |v| v.is_object()) {
                ag_provider["options"] = serde_json::json!({});
            }
            if let Some(options) = ag_provider
                .get_mut("options")
                .and_then(|o| o.as_object_mut())
            {
                options.insert("baseURL".to_string(), Value::String(normalized_url));
                options.insert("apiKey".to_string(), Value::String(api_key.to_string()));
            }

            // Always update models from proxy (reflects current proxy model list)
            if !fetched_models.is_empty() {
                if let Some(obj) = ag_provider.as_object_mut() {
                    obj.insert("models".to_string(), Value::Object(fetched_models));
                }
            }
        }
    }

    let content = utils::to_json_pretty(&config).map_err(|e| e.to_string())?;
    utils::atomic_write(&config_path, &content).map_err(|e| e.to_string())
}

pub fn restore_opencode_config() -> Result<(), String> {
    let config_path =
        get_config_path().ok_or_else(|| "Failed to get OpenCode config directory".to_string())?;

    let backup_path =
        config_path.with_file_name(format!("{}{}", OPENCODE_CONFIG_FILE, BACKUP_SUFFIX));
    if backup_path.exists() {
        // Atomic rename replaces the target file directly — no intermediate delete needed.
        fs::rename(&backup_path, &config_path)
            .map_err(|e| format!("Failed to restore config: {}", e))?;
        Ok(())
    } else {
        Err("No backup file found".to_string())
    }
}

pub fn read_opencode_config_content() -> Result<String, String> {
    let config_path =
        get_config_path().ok_or_else(|| "Failed to get OpenCode config directory".to_string())?;

    if !config_path.exists() {
        return Err(format!("Config file does not exist: {:?}", config_path));
    }

    fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))
}

pub fn write_opencode_config_content(content: &str) -> Result<(), String> {
    let config_path = get_config_path().ok_or_else(|| "Config path not found".to_string())?;
    serde_json::from_str::<serde_json::Value>(content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_base_url() {
        assert_eq!(
            normalize_base_url("http://localhost:3000"),
            "http://localhost:3000/v1"
        );
        assert_eq!(
            normalize_base_url("http://localhost:3000/"),
            "http://localhost:3000/v1"
        );
        assert_eq!(
            normalize_base_url("http://localhost:3000/v1"),
            "http://localhost:3000/v1"
        );
        assert_eq!(
            normalize_base_url("http://localhost:3000/v1/"),
            "http://localhost:3000/v1"
        );
        assert_eq!(normalize_base_url("  http://x.com  "), "http://x.com/v1");
    }
}
