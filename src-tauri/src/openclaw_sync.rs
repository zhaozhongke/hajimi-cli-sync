use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::utils;

const CONFIG_FILE: &str = "openclaw.json";
const BACKUP_SUFFIX: &str = ".antigravity.bak";
const PROVIDER_ID: &str = "hajimi";

fn get_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".openclaw"))
}

fn get_config_path() -> Option<PathBuf> {
    get_config_dir().map(|dir| dir.join(CONFIG_FILE))
}

pub fn check_openclaw_installed() -> (bool, Option<String>) {
    match utils::resolve_executable("openclaw") {
        Some(path) => {
            let version = utils::get_cli_version(&path);
            (true, version.or_else(|| Some("detected".to_string())))
        }
        None => {
            // Also check if config dir exists (installed but not in PATH)
            let has_config = get_config_dir().map_or(false, |d| d.exists());
            if has_config {
                (true, Some("detected".to_string()))
            } else {
                (false, None)
            }
        }
    }
}

pub fn get_sync_status(proxy_url: &str) -> (bool, bool, Option<String>) {
    let config_path = match get_config_path() {
        Some(p) => p,
        None => return (false, false, None),
    };

    let backup_path = config_path.with_file_name(format!("{}{}", CONFIG_FILE, BACKUP_SUFFIX));
    let has_backup = backup_path.exists();

    if !config_path.exists() {
        return (false, has_backup, None);
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return (false, has_backup, None),
    };

    // OpenClaw uses JSON5 but serde_json can parse standard JSON subset
    let json: Value = serde_json::from_str(&content).unwrap_or_default();

    let current_url = json
        .get("models")
        .and_then(|m| m.get("providers"))
        .and_then(|p| p.get(PROVIDER_ID))
        .and_then(|h| h.get("baseUrl"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let is_synced = current_url
        .as_deref()
        .map_or(false, |u| urls_match(u, proxy_url));

    (is_synced, has_backup, current_url)
}

fn urls_match(a: &str, b: &str) -> bool {
    let normalize = |s: &str| {
        let trimmed = s.trim().trim_end_matches('/');
        if trimmed.ends_with("/v1") {
            trimmed.to_string()
        } else {
            format!("{}/v1", trimmed)
        }
    };
    normalize(a) == normalize(b)
}

fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{}/v1", trimmed)
    }
}

/// Fetch models from proxy and build OpenClaw models array format.
async fn fetch_models_for_openclaw(
    base_url: &str,
    api_key: &str,
) -> Vec<Value> {
    let models_url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let resp = match client
        .get(&models_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return vec![],
    };

    let body: Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let mut models = Vec::new();
    if let Some(data) = body.get("data").and_then(|v| v.as_array()) {
        for item in data {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                let is_reasoning = id.contains("thinking") || id.contains("pro");
                let is_claude = id.contains("claude");
                let is_gemini = id.contains("gemini");
                let is_image = id.contains("image");

                let context_window: u64 = if is_claude { 200_000 } else if is_gemini { 1_048_576 } else { 128_000 };
                let max_tokens: u64 = if is_claude { 64_000 } else { 65_536 };

                let mut input_modalities = vec!["text"];
                if is_claude || is_gemini {
                    input_modalities.push("image");
                }

                let model = serde_json::json!({
                    "id": id,
                    "name": id,
                    "reasoning": is_reasoning,
                    "input": input_modalities,
                    "cost": { "input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0 },
                    "contextWindow": context_window,
                    "maxTokens": max_tokens,
                });

                // Skip pure image generation models for coding agent use
                if !is_image {
                    models.push(model);
                }
            }
        }
    }
    models
}

pub async fn sync_openclaw_config(proxy_url: &str, api_key: &str) -> Result<(), String> {
    let config_path = get_config_path().ok_or_else(|| {
        "Failed to determine OpenClaw config directory".to_string()
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

    let normalized_url = normalize_base_url(proxy_url);

    // Fetch models from proxy
    let fetched_models = fetch_models_for_openclaw(&normalized_url, api_key).await;

    // Ensure models.providers path exists
    if !config.get("models").map_or(false, |v| v.is_object()) {
        config["models"] = serde_json::json!({});
    }
    if !config["models"].get("providers").map_or(false, |v| v.is_object()) {
        config["models"]["providers"] = serde_json::json!({});
    }

    // Set merge mode to keep built-in providers
    if config["models"].get("mode").is_none() {
        config["models"]["mode"] = Value::String("merge".to_string());
    }

    // Build hajimi provider
    let mut provider = serde_json::json!({
        "baseUrl": normalized_url,
        "apiKey": api_key,
        "api": "openai-completions",
    });

    if !fetched_models.is_empty() {
        provider["models"] = Value::Array(fetched_models);
    }

    // Insert/update hajimi provider
    if let Some(providers) = config["models"]
        .get_mut("providers")
        .and_then(|p| p.as_object_mut())
    {
        providers.insert(PROVIDER_ID.to_string(), provider);
    }

    let content = utils::to_json_pretty(&config).map_err(|e| e.to_string())?;
    utils::atomic_write(&config_path, &content).map_err(|e| e.to_string())
}

pub fn restore_openclaw_config() -> Result<(), String> {
    let config_path =
        get_config_path().ok_or_else(|| "Failed to get OpenClaw config directory".to_string())?;

    let backup_path =
        config_path.with_file_name(format!("{}{}", CONFIG_FILE, BACKUP_SUFFIX));
    if backup_path.exists() {
        if config_path.exists() {
            fs::remove_file(&config_path)
                .map_err(|e| format!("Failed to remove config: {}", e))?;
        }
        fs::rename(&backup_path, &config_path)
            .map_err(|e| format!("Failed to restore config: {}", e))?;
        Ok(())
    } else {
        Err("No backup file found".to_string())
    }
}

pub fn read_openclaw_config_content() -> Result<String, String> {
    let config_path =
        get_config_path().ok_or_else(|| "Failed to get OpenClaw config directory".to_string())?;

    if !config_path.exists() {
        return Err(format!("Config file does not exist: {:?}", config_path));
    }

    fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))
}

pub fn write_openclaw_config_content(content: &str) -> Result<(), String> {
    let config_path = get_config_path().ok_or_else(|| "Config path not found".to_string())?;
    serde_json::from_str::<serde_json::Value>(content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urls_match() {
        assert!(urls_match("https://example.com/v1", "https://example.com/v1"));
        assert!(urls_match("https://example.com/v1/", "https://example.com/v1"));
        assert!(urls_match("https://example.com", "https://example.com/v1"));
        assert!(!urls_match("https://a.com", "https://b.com"));
    }

    #[test]
    fn test_normalize_base_url() {
        assert_eq!(normalize_base_url("https://x.com"), "https://x.com/v1");
        assert_eq!(normalize_base_url("https://x.com/v1"), "https://x.com/v1");
        assert_eq!(normalize_base_url("https://x.com/v1/"), "https://x.com/v1");
    }
}
