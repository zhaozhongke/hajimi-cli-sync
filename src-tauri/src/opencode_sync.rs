use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::utils;

const OPENCODE_CONFIG_FILE: &str = "opencode.json";
const BACKUP_SUFFIX: &str = ".antigravity.bak";
const PROVIDER_ID: &str = "hajimi";

/// Model definition with metadata
#[derive(Debug, Clone)]
struct ModelDef {
    id: &'static str,
    name: &'static str,
    context_limit: u32,
    output_limit: u32,
    input_modalities: &'static [&'static str],
    output_modalities: &'static [&'static str],
    reasoning: bool,
}

fn build_model_catalog() -> Vec<ModelDef> {
    vec![
        ModelDef { id: "claude-sonnet-4-5", name: "Claude Sonnet 4.5", context_limit: 200_000, output_limit: 64_000, input_modalities: &["text", "image", "pdf"], output_modalities: &["text"], reasoning: false },
        ModelDef { id: "claude-sonnet-4-5-thinking", name: "Claude Sonnet 4.5 Thinking", context_limit: 200_000, output_limit: 64_000, input_modalities: &["text", "image", "pdf"], output_modalities: &["text"], reasoning: true },
        ModelDef { id: "claude-opus-4-5-thinking", name: "Claude Opus 4.5 Thinking", context_limit: 200_000, output_limit: 64_000, input_modalities: &["text", "image", "pdf"], output_modalities: &["text"], reasoning: true },
        ModelDef { id: "gemini-3-pro-high", name: "Gemini 3 Pro High", context_limit: 1_048_576, output_limit: 65_535, input_modalities: &["text", "image", "pdf"], output_modalities: &["text", "image"], reasoning: true },
        ModelDef { id: "gemini-3-pro-low", name: "Gemini 3 Pro Low", context_limit: 1_048_576, output_limit: 65_535, input_modalities: &["text", "image", "pdf"], output_modalities: &["text", "image"], reasoning: true },
        ModelDef { id: "gemini-3-flash", name: "Gemini 3 Flash", context_limit: 1_048_576, output_limit: 65_536, input_modalities: &["text", "image", "pdf"], output_modalities: &["text"], reasoning: true },
        ModelDef { id: "gemini-2.5-flash", name: "Gemini 2.5 Flash", context_limit: 1_048_576, output_limit: 65_536, input_modalities: &["text", "image", "pdf"], output_modalities: &["text"], reasoning: false },
        ModelDef { id: "gemini-2.5-pro", name: "Gemini 2.5 Pro", context_limit: 1_048_576, output_limit: 65_536, input_modalities: &["text", "image", "pdf"], output_modalities: &["text"], reasoning: true },
    ]
}

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
    dirs::home_dir().map(|h| h.join(".config").join("opencode"))
}

fn get_config_path() -> Option<PathBuf> {
    get_opencode_dir().map(|dir| dir.join(OPENCODE_CONFIG_FILE))
}

pub fn check_opencode_installed() -> (bool, Option<String>) {
    match utils::resolve_executable("opencode") {
        Some(path) => {
            let version = utils::get_cli_version(&path);
            // If resolved but version failed, still report as installed
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

    let backup_path = config_path.with_file_name(format!("{}{}", OPENCODE_CONFIG_FILE, BACKUP_SUFFIX));
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

fn build_model_json(model_def: &ModelDef) -> Value {
    let mut model_obj = serde_json::Map::new();
    model_obj.insert("name".to_string(), Value::String(model_def.name.to_string()));
    model_obj.insert("limit".to_string(), serde_json::json!({
        "context": model_def.context_limit,
        "output": model_def.output_limit,
    }));
    model_obj.insert("modalities".to_string(), serde_json::json!({
        "input": model_def.input_modalities,
        "output": model_def.output_modalities,
    }));
    if model_def.reasoning {
        model_obj.insert("reasoning".to_string(), Value::Bool(true));
    }
    Value::Object(model_obj)
}

pub fn sync_opencode_config(proxy_url: &str, api_key: &str) -> Result<(), String> {
    let config_path = get_config_path()
        .ok_or_else(|| "Failed to get OpenCode config directory (home dir not found)".to_string())?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
    }

    utils::create_backup(&config_path, BACKUP_SUFFIX).map_err(|e| e.to_string())?;

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

    // Ensure provider object
    if !config.get("provider").map_or(false, |v| v.is_object()) {
        config["provider"] = serde_json::json!({});
    }

    if let Some(provider) = config.get_mut("provider").and_then(|p| p.as_object_mut()) {
        if !provider.get(PROVIDER_ID).map_or(false, |v| v.is_object()) {
            provider.insert(PROVIDER_ID.to_string(), serde_json::json!({}));
        }
        if let Some(ag_provider) = provider.get_mut(PROVIDER_ID) {
            if let Some(obj) = ag_provider.as_object_mut() {
                obj.insert("npm".to_string(), Value::String("@ai-sdk/anthropic".to_string()));
                obj.insert("name".to_string(), Value::String("Hajimi".to_string()));
            }

            if !ag_provider.get("options").map_or(false, |v| v.is_object()) {
                ag_provider["options"] = serde_json::json!({});
            }
            if let Some(options) = ag_provider.get_mut("options").and_then(|o| o.as_object_mut()) {
                options.insert("baseURL".to_string(), Value::String(normalized_url));
                options.insert("apiKey".to_string(), Value::String(api_key.to_string()));
            }

            if !ag_provider.get("models").map_or(false, |v| v.is_object()) {
                ag_provider["models"] = serde_json::json!({});
            }
            if let Some(models) = ag_provider.get_mut("models").and_then(|m| m.as_object_mut()) {
                for model_def in &build_model_catalog() {
                    models.insert(model_def.id.to_string(), build_model_json(model_def));
                }
            }
        }
    }

    let content = utils::to_json_pretty(&config).map_err(|e| e.to_string())?;
    utils::atomic_write(&config_path, &content).map_err(|e| e.to_string())
}

pub fn restore_opencode_config() -> Result<(), String> {
    let config_path = get_config_path()
        .ok_or_else(|| "Failed to get OpenCode config directory".to_string())?;

    let backup_path = config_path.with_file_name(format!("{}{}", OPENCODE_CONFIG_FILE, BACKUP_SUFFIX));
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

pub fn read_opencode_config_content() -> Result<String, String> {
    let config_path = get_config_path()
        .ok_or_else(|| "Failed to get OpenCode config directory".to_string())?;

    if !config_path.exists() {
        return Err(format!("Config file does not exist: {:?}", config_path));
    }

    fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_base_url() {
        assert_eq!(normalize_base_url("http://localhost:3000"), "http://localhost:3000/v1");
        assert_eq!(normalize_base_url("http://localhost:3000/"), "http://localhost:3000/v1");
        assert_eq!(normalize_base_url("http://localhost:3000/v1"), "http://localhost:3000/v1");
        assert_eq!(normalize_base_url("http://localhost:3000/v1/"), "http://localhost:3000/v1");
        assert_eq!(normalize_base_url("  http://x.com  "), "http://x.com/v1");
    }

    #[test]
    fn test_build_model_catalog_not_empty() {
        let catalog = build_model_catalog();
        assert!(!catalog.is_empty());
        assert!(catalog.iter().any(|m| m.id == "claude-sonnet-4-5"));
    }

    #[test]
    fn test_build_model_json_structure() {
        let catalog = build_model_catalog();
        let model = &catalog[0];
        let json = build_model_json(model);
        assert!(json.get("name").is_some());
        assert!(json.get("limit").is_some());
        assert!(json.get("modalities").is_some());
    }
}
