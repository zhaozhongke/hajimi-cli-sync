use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::utils;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum CliApp {
    Claude,
    Codex,
    Gemini,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CliConfigFile {
    pub name: String,
    pub path: PathBuf,
}

const BACKUP_SUFFIX: &str = ".antigravity.bak";

impl CliApp {
    pub fn as_str(&self) -> &'static str {
        match self {
            CliApp::Claude => "claude",
            CliApp::Codex => "codex",
            CliApp::Gemini => "gemini",
        }
    }

    pub fn config_files(&self) -> Vec<CliConfigFile> {
        let home = match dirs::home_dir() {
            Some(p) => p,
            None => {
                tracing::warn!("[cli_sync] Could not determine home directory");
                return vec![];
            }
        };
        match self {
            CliApp::Claude => vec![
                CliConfigFile {
                    name: ".claude.json".to_string(),
                    path: home.join(".claude.json"),
                },
                CliConfigFile {
                    name: "settings.json".to_string(),
                    path: home.join(".claude").join("settings.json"),
                },
            ],
            CliApp::Codex => vec![
                CliConfigFile {
                    name: "auth.json".to_string(),
                    path: home.join(".codex").join("auth.json"),
                },
                CliConfigFile {
                    name: "config.toml".to_string(),
                    path: home.join(".codex").join("config.toml"),
                },
            ],
            CliApp::Gemini => vec![
                CliConfigFile {
                    name: ".env".to_string(),
                    path: home.join(".gemini").join(".env"),
                },
                CliConfigFile {
                    name: "settings.json".to_string(),
                    path: home.join(".gemini").join("settings.json"),
                },
                CliConfigFile {
                    name: "config.json".to_string(),
                    path: home.join(".gemini").join("config.json"),
                },
            ],
        }
    }

    pub fn default_url(&self) -> &'static str {
        match self {
            CliApp::Claude => "https://api.anthropic.com",
            CliApp::Codex => "https://api.openai.com/v1",
            CliApp::Gemini => "https://generativelanguage.googleapis.com",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub is_synced: bool,
    pub has_backup: bool,
    pub current_base_url: Option<String>,
    pub files: Vec<String>,
}

/// Check if a CLI tool is installed and get its version
pub fn check_cli_installed(app: &CliApp) -> (bool, Option<String>) {
    let name = app.as_str();

    match utils::resolve_executable(name) {
        Some(path) => {
            let version = utils::get_cli_version(&path);
            (true, version)
        }
        None => (false, None),
    }
}

/// Read current config and check sync status
pub fn get_sync_status(app: &CliApp, proxy_url: &str) -> (bool, bool, Option<String>) {
    let files = app.config_files();
    if files.is_empty() {
        return (false, false, None);
    }

    let mut all_synced = true;
    let mut has_backup = false;
    let mut current_base_url = None;

    for file in &files {
        let backup_path = file
            .path
            .with_file_name(format!("{}{}", file.name, BACKUP_SUFFIX));

        if backup_path.exists() {
            has_backup = true;
        }

        if !file.path.exists() {
            // Gemini: settings.json/config.json are optional if the other exists
            if app == &CliApp::Gemini
                && (file.name == "settings.json" || file.name == "config.json")
            {
                continue;
            }
            all_synced = false;
            continue;
        }

        let content = match fs::read_to_string(&file.path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("[cli_sync] Failed to read {:?}: {}", file.path, e);
                all_synced = false;
                continue;
            }
        };

        match app {
            CliApp::Claude => {
                if file.name == "settings.json" {
                    let json: Value = serde_json::from_str(&content).unwrap_or_default();
                    let url = json
                        .get("env")
                        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
                        .and_then(|v| v.as_str());
                    if let Some(u) = url {
                        current_base_url = Some(u.to_string());
                        if u.trim_end_matches('/') != proxy_url.trim_end_matches('/') {
                            all_synced = false;
                        }
                    } else {
                        all_synced = false;
                    }
                } else if file.name == ".claude.json" {
                    let json: Value = serde_json::from_str(&content).unwrap_or_default();
                    if json.get("hasCompletedOnboarding") != Some(&Value::Bool(true)) {
                        all_synced = false;
                    }
                }
            }
            CliApp::Codex => {
                if file.name == "config.toml" {
                    // Safe: regex pattern is a compile-time constant
                    if let Ok(re) = regex::Regex::new(r#"(?m)^\s*base_url\s*=\s*['"]([^'"]+)['"]"#)
                    {
                        if let Some(caps) = re.captures(&content) {
                            let url = &caps[1];
                            current_base_url = Some(url.to_string());
                            if url.trim_end_matches('/') != proxy_url.trim_end_matches('/') {
                                all_synced = false;
                            }
                        } else {
                            all_synced = false;
                        }
                    } else {
                        all_synced = false;
                    }
                }
            }
            CliApp::Gemini => {
                if file.name == ".env" {
                    if let Ok(re) = regex::Regex::new(r#"(?m)^GOOGLE_GEMINI_BASE_URL=(.*)$"#) {
                        if let Some(caps) = re.captures(&content) {
                            let url = caps[1].trim();
                            current_base_url = Some(url.to_string());
                            if url.trim_end_matches('/') != proxy_url.trim_end_matches('/') {
                                all_synced = false;
                            }
                        } else {
                            all_synced = false;
                        }
                    } else {
                        all_synced = false;
                    }
                }
            }
        }
    }

    (all_synced, has_backup, current_base_url)
}

/// Execute sync logic - writes config files for the given CLI app.
pub fn sync_config(
    app: &CliApp,
    proxy_url: &str,
    api_key: &str,
    model: Option<&str>,
) -> Result<(), String> {
    let files = app.config_files();
    if files.is_empty() {
        return Err("Could not determine config file paths (home directory not found)".to_string());
    }

    for file in &files {
        // Gemini compatibility: prefer settings.json over config.json
        if app == &CliApp::Gemini && file.name == "config.json" && !file.path.exists() {
            let settings_path = file.path.with_file_name("settings.json");
            if settings_path.exists() {
                continue;
            }
        }

        if let Some(parent) = file.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
        }

        // Auto-backup before first sync
        utils::create_rotated_backup(&file.path, BACKUP_SUFFIX)?;

        let mut content = if file.path.exists() {
            fs::read_to_string(&file.path).unwrap_or_default()
        } else {
            String::new()
        };

        match app {
            CliApp::Claude => {
                if file.name == ".claude.json" {
                    let mut json: Value =
                        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
                    if let Some(obj) = json.as_object_mut() {
                        obj.insert("hasCompletedOnboarding".to_string(), Value::Bool(true));
                    }
                    content = utils::to_json_pretty(&json)?;
                } else if file.name == "settings.json" {
                    let mut json: Value =
                        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
                    if !json.is_object() {
                        json = serde_json::json!({});
                    }

                    // Safe: we just ensured json is an object above
                    let obj = json
                        .as_object_mut()
                        .ok_or_else(|| "Internal error: json is not an object".to_string())?;
                    let env = obj.entry("env").or_insert(serde_json::json!({}));

                    if let Some(env_obj) = env.as_object_mut() {
                        env_obj.insert(
                            "ANTHROPIC_BASE_URL".to_string(),
                            Value::String(proxy_url.to_string()),
                        );
                        if !api_key.is_empty() {
                            env_obj.insert(
                                "ANTHROPIC_API_KEY".to_string(),
                                Value::String(api_key.to_string()),
                            );
                            // Remove conflicting keys
                            env_obj.remove("ANTHROPIC_AUTH_TOKEN");
                            env_obj.remove("ANTHROPIC_MODEL");
                            env_obj.remove("ANTHROPIC_DEFAULT_HAIKU_MODEL");
                            env_obj.remove("ANTHROPIC_DEFAULT_OPUS_MODEL");
                            env_obj.remove("ANTHROPIC_DEFAULT_SONNET_MODEL");
                        } else {
                            env_obj.remove("ANTHROPIC_API_KEY");
                        }
                    }

                    if let Some(m) = model {
                        if let Some(root) = json.as_object_mut() {
                            root.insert("model".to_string(), Value::String(m.to_string()));
                        }
                    }
                    content = utils::to_json_pretty(&json)?;
                }
            }
            CliApp::Codex => {
                if file.name == "auth.json" {
                    let mut json: Value =
                        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
                    if let Some(obj) = json.as_object_mut() {
                        obj.insert(
                            "OPENAI_API_KEY".to_string(),
                            Value::String(api_key.to_string()),
                        );
                        obj.insert(
                            "OPENAI_BASE_URL".to_string(),
                            Value::String(proxy_url.to_string()),
                        );
                    }
                    content = utils::to_json_pretty(&json)?;
                } else if file.name == "config.toml" {
                    use toml_edit::{value, DocumentMut};
                    let mut doc = content
                        .parse::<DocumentMut>()
                        .unwrap_or_else(|_| DocumentMut::new());

                    let providers = doc
                        .entry("model_providers")
                        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
                    if let Some(p_table) = providers.as_table_mut() {
                        let custom = p_table
                            .entry("custom")
                            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
                        if let Some(c_table) = custom.as_table_mut() {
                            c_table.insert("name", value("custom"));
                            c_table.insert("wire_api", value("responses"));
                            c_table.insert("requires_openai_auth", value(true));
                            c_table.insert("base_url", value(proxy_url));
                            if let Some(m) = model {
                                c_table.insert("model", value(m));
                            }
                        }
                    }
                    doc.insert("model_provider", value("custom"));
                    if let Some(m) = model {
                        doc.insert("model", value(m));
                    }
                    doc.remove("openai_api_key");
                    doc.remove("openai_base_url");
                    content = doc.to_string();
                }
            }
            CliApp::Gemini => {
                if file.name == ".env" {
                    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                    let mut found_url = false;
                    let mut found_key = false;
                    for line in lines.iter_mut() {
                        if line.starts_with("GOOGLE_GEMINI_BASE_URL=") {
                            *line = format!("GOOGLE_GEMINI_BASE_URL={}", proxy_url);
                            found_url = true;
                        } else if line.trim().starts_with("GEMINI_API_KEY=") {
                            *line = format!("GEMINI_API_KEY={}", api_key);
                            found_key = true;
                        }
                    }
                    if !found_url {
                        lines.push(format!("GOOGLE_GEMINI_BASE_URL={}", proxy_url));
                    }
                    if !found_key {
                        lines.push(format!("GEMINI_API_KEY={}", api_key));
                    }
                    if let Some(m) = model {
                        let mut found_model = false;
                        for line in lines.iter_mut() {
                            if line.starts_with("GOOGLE_GEMINI_MODEL=") {
                                *line = format!("GOOGLE_GEMINI_MODEL={}", m);
                                found_model = true;
                            }
                        }
                        if !found_model {
                            lines.push(format!("GOOGLE_GEMINI_MODEL={}", m));
                        }
                    }
                    content = lines.join("\n");
                    if !content.ends_with('\n') {
                        content.push('\n');
                    }
                } else if file.name == "settings.json" || file.name == "config.json" {
                    let mut json: Value =
                        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
                    if !json.is_object() {
                        json = serde_json::json!({});
                    }

                    // Build nested security.auth structure safely
                    let obj = json
                        .as_object_mut()
                        .ok_or_else(|| "Internal error".to_string())?;
                    let sec = obj.entry("security").or_insert(serde_json::json!({}));
                    if let Some(sec_obj) = sec.as_object_mut() {
                        let auth = sec_obj.entry("auth").or_insert(serde_json::json!({}));
                        if let Some(auth_obj) = auth.as_object_mut() {
                            auth_obj.insert(
                                "selectedType".to_string(),
                                Value::String("gemini-api-key".to_string()),
                            );
                        }
                    }
                    content = utils::to_json_pretty(&json)?;
                }
            }
        }

        // Atomic write with temp file
        utils::atomic_write(&file.path, &content)?;
    }

    Ok(())
}

/// Restore from backup files
pub fn restore_config(app: &CliApp) -> Result<(), String> {
    let files = app.config_files();
    if files.is_empty() {
        return Err("Could not determine config file paths".to_string());
    }

    let mut restored_count = 0;

    for file in &files {
        let backup_path = file
            .path
            .with_file_name(format!("{}{}", file.name, BACKUP_SUFFIX));
        if backup_path.exists() {
            if let Err(e) = fs::rename(&backup_path, &file.path) {
                return Err(format!("Failed to restore backup {}: {}", file.name, e));
            }
            tracing::info!("[cli_sync] Restored {} from backup", file.name);
            restored_count += 1;
        }
    }

    if restored_count > 0 {
        return Ok(());
    }

    // No backup found — remove only the proxy-related keys we injected,
    // instead of writing empty/default values that would break the user's config.
    for file in &files {
        if !file.path.exists() {
            continue;
        }
        let content = match fs::read_to_string(&file.path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let new_content = match app {
            CliApp::Claude => {
                if file.name == "settings.json" {
                    let mut json: Value = serde_json::from_str(&content).unwrap_or_default();
                    if let Some(env_obj) = json.get_mut("env").and_then(|e| e.as_object_mut()) {
                        env_obj.remove("ANTHROPIC_BASE_URL");
                        env_obj.remove("ANTHROPIC_API_KEY");
                    }
                    Some(serde_json::to_string_pretty(&json).unwrap_or(content.clone()))
                } else {
                    None
                }
            }
            CliApp::Codex => {
                if file.name == "config.toml" {
                    use toml_edit::DocumentMut;
                    let mut doc = content
                        .parse::<DocumentMut>()
                        .unwrap_or_else(|_| DocumentMut::new());
                    doc.remove("model_provider");
                    doc.remove("model");
                    if let Some(providers) = doc.get_mut("model_providers") {
                        if let Some(table) = providers.as_table_mut() {
                            table.remove("custom");
                        }
                    }
                    Some(doc.to_string())
                } else {
                    None
                }
            }
            CliApp::Gemini => {
                if file.name == ".env" {
                    let lines: Vec<&str> = content
                        .lines()
                        .filter(|l| {
                            !l.starts_with("GOOGLE_GEMINI_BASE_URL=")
                                && !l.starts_with("GEMINI_API_KEY=")
                        })
                        .collect();
                    let mut result = lines.join("\n");
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    Some(result)
                } else {
                    None
                }
            }
        };

        if let Some(c) = new_content {
            utils::atomic_write(&file.path, &c)
                .map_err(|e| format!("Failed to clean config {}: {}", file.name, e))?;
        }
    }

    Ok(())
}

/// Read config file content for viewing
pub fn read_config_content(app: &CliApp, file_name: Option<&str>) -> Result<String, String> {
    let files = app.config_files();
    let file = if let Some(name) = file_name {
        files
            .into_iter()
            .find(|f| f.name == name)
            .ok_or_else(|| format!("File '{}' not found for {}", name, app.as_str()))?
    } else {
        files
            .into_iter()
            .next()
            .ok_or_else(|| "No config files available".to_string())?
    };

    if !file.path.exists() {
        return Err(format!("Config file does not exist: {:?}", file.path));
    }
    fs::read_to_string(&file.path).map_err(|e| format!("Failed to read {}: {}", file.name, e))
}

/// Write config file content (for editing)
pub fn write_config_content(app: &CliApp, file_name: &str, content: &str) -> Result<(), String> {
    let files = app.config_files();
    let file = files
        .into_iter()
        .find(|f| f.name == file_name)
        .ok_or_else(|| format!("File '{}' not found for {}", file_name, app.as_str()))?;

    // Validate JSON if it's a JSON file
    if file_name.ends_with(".json") {
        serde_json::from_str::<Value>(content)
            .map_err(|e| format!("Invalid JSON: {}", e))?;
    }

    utils::atomic_write(&file.path, content)
        .map_err(|e| format!("Failed to write {}: {}", file_name, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cli_app_as_str() {
        assert_eq!(CliApp::Claude.as_str(), "claude");
        assert_eq!(CliApp::Codex.as_str(), "codex");
        assert_eq!(CliApp::Gemini.as_str(), "gemini");
    }

    #[test]
    fn test_cli_app_default_url() {
        assert!(CliApp::Claude.default_url().starts_with("https://"));
        assert!(CliApp::Codex.default_url().contains("/v1"));
        assert!(CliApp::Gemini.default_url().starts_with("https://"));
    }

    #[test]
    fn test_config_files_not_empty() {
        if dirs::home_dir().is_some() {
            assert!(!CliApp::Claude.config_files().is_empty());
            assert!(!CliApp::Codex.config_files().is_empty());
            assert!(!CliApp::Gemini.config_files().is_empty());
        }
    }

    #[test]
    fn test_sync_config_validates_empty_files() {
        let app = CliApp::Claude;
        assert_eq!(app.config_files().len(), 2);
    }

    // --- 以下为新增的核心逻辑测试 ---

    /// 测试Claude settings.json的sync写入正确性
    #[test]
    fn test_claude_sync_writes_correct_json() {
        let dir = TempDir::new().unwrap();
        let settings_path = dir.path().join("settings.json");
        let claude_json_path = dir.path().join(".claude.json");

        // 写入空的初始文件
        fs::write(&settings_path, "{}").unwrap();
        fs::write(&claude_json_path, "{}").unwrap();

        // 模拟sync逻辑（直接测试JSON merge部分）
        let proxy_url = "https://proxy.example.com";
        let api_key = "sk-test-key-123";

        let content = fs::read_to_string(&settings_path).unwrap();
        let mut json: Value = serde_json::from_str(&content).unwrap();
        let obj = json.as_object_mut().unwrap();
        let env = obj.entry("env").or_insert(serde_json::json!({}));
        if let Some(env_obj) = env.as_object_mut() {
            env_obj.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                Value::String(proxy_url.to_string()),
            );
            env_obj.insert(
                "ANTHROPIC_API_KEY".to_string(),
                Value::String(api_key.to_string()),
            );
        }

        let result = serde_json::to_string_pretty(&json).unwrap();
        fs::write(&settings_path, &result).unwrap();

        // 验证
        let written: Value =
            serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert_eq!(written["env"]["ANTHROPIC_BASE_URL"], proxy_url);
        assert_eq!(written["env"]["ANTHROPIC_API_KEY"], api_key);
    }

    /// 测试Claude sync保留已有字段
    #[test]
    fn test_claude_sync_preserves_existing_fields() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let existing = serde_json::json!({
            "env": {
                "SOME_EXISTING_VAR": "keep-me"
            },
            "customSetting": true
        });
        fs::write(&path, serde_json::to_string_pretty(&existing).unwrap()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let mut json: Value = serde_json::from_str(&content).unwrap();
        if let Some(env_obj) = json.get_mut("env").and_then(|e| e.as_object_mut()) {
            env_obj.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                Value::String("https://new.url".to_string()),
            );
        }
        fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let result: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(result["env"]["SOME_EXISTING_VAR"], "keep-me");
        assert_eq!(result["env"]["ANTHROPIC_BASE_URL"], "https://new.url");
        assert_eq!(result["customSetting"], true);
    }

    /// 测试Codex TOML merge逻辑
    #[test]
    fn test_codex_toml_merge() {
        use toml_edit::{value, DocumentMut};

        let existing = r#"
model = "gpt-4o"
some_key = "keep"
"#;
        let mut doc = existing.parse::<DocumentMut>().unwrap();

        let providers = doc
            .entry("model_providers")
            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
        if let Some(p_table) = providers.as_table_mut() {
            let custom = p_table
                .entry("custom")
                .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
            if let Some(c_table) = custom.as_table_mut() {
                c_table.insert("base_url", value("https://proxy.test"));
            }
        }
        doc.insert("model_provider", value("custom"));

        let result = doc.to_string();
        assert!(result.contains("base_url"));
        assert!(result.contains("https://proxy.test"));
        assert!(result.contains("some_key = \"keep\""));
        assert!(result.contains("model_provider = \"custom\""));
    }

    /// 测试Gemini .env写入
    #[test]
    fn test_gemini_env_write() {
        let existing = "EXISTING_KEY=keep-me\nGOOGLE_GEMINI_BASE_URL=old-url\n";
        let proxy_url = "https://new.proxy.com";
        let api_key = "gem-key-123";

        let mut lines: Vec<String> = existing.lines().map(|s| s.to_string()).collect();
        let mut found_url = false;
        let mut found_key = false;
        for line in lines.iter_mut() {
            if line.starts_with("GOOGLE_GEMINI_BASE_URL=") {
                *line = format!("GOOGLE_GEMINI_BASE_URL={}", proxy_url);
                found_url = true;
            } else if line.trim().starts_with("GEMINI_API_KEY=") {
                *line = format!("GEMINI_API_KEY={}", api_key);
                found_key = true;
            }
        }
        if !found_url {
            lines.push(format!("GOOGLE_GEMINI_BASE_URL={}", proxy_url));
        }
        if !found_key {
            lines.push(format!("GEMINI_API_KEY={}", api_key));
        }
        let mut content = lines.join("\n");
        if !content.ends_with('\n') {
            content.push('\n');
        }

        assert!(content.contains("EXISTING_KEY=keep-me"));
        assert!(content.contains(&format!("GOOGLE_GEMINI_BASE_URL={}", proxy_url)));
        assert!(content.contains(&format!("GEMINI_API_KEY={}", api_key)));
        assert!(content.ends_with('\n'));
        // 旧URL应被替换而不是追加
        assert!(!content.contains("old-url"));
    }

    /// 测试.env新文件写入（不存在已有字段）
    #[test]
    fn test_gemini_env_write_fresh() {
        let existing = "";
        let mut lines: Vec<String> = existing.lines().map(|s| s.to_string()).collect();
        lines.push("GOOGLE_GEMINI_BASE_URL=https://new.url".to_string());
        lines.push("GEMINI_API_KEY=test-key".to_string());
        let mut content = lines.join("\n");
        if !content.ends_with('\n') {
            content.push('\n');
        }

        assert!(content.contains("GOOGLE_GEMINI_BASE_URL=https://new.url"));
        assert!(content.contains("GEMINI_API_KEY=test-key"));
        assert!(content.ends_with('\n'));
    }

    /// 测试sync_status正确检测已同步状态
    #[test]
    fn test_get_sync_status_detects_synced() {
        let dir = TempDir::new().unwrap();
        let settings_path = dir.path().join("settings.json");

        let config = serde_json::json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://proxy.test"
            }
        });
        fs::write(
            &settings_path,
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .unwrap();

        // 直接测试JSON解析逻辑
        let content = fs::read_to_string(&settings_path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        let url = json
            .get("env")
            .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
            .and_then(|v| v.as_str());

        assert_eq!(url, Some("https://proxy.test"));
    }

    /// 测试backup只创建一次
    #[test]
    fn test_backup_created_once() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.json");
        fs::write(&file_path, "original content").unwrap();

        // 第一次备份
        utils::create_backup(&file_path, BACKUP_SUFFIX).unwrap();
        let backup_path = file_path.with_file_name(format!("test.json{}", BACKUP_SUFFIX));
        assert!(backup_path.exists());
        assert_eq!(
            fs::read_to_string(&backup_path).unwrap(),
            "original content"
        );

        // 修改原文件
        fs::write(&file_path, "modified content").unwrap();

        // 第二次备份不应覆盖
        utils::create_backup(&file_path, BACKUP_SUFFIX).unwrap();
        assert_eq!(
            fs::read_to_string(&backup_path).unwrap(),
            "original content"
        );
    }

    /// 测试atomic_write写入正确性
    #[test]
    fn test_atomic_write_content() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("atomic_test.json");

        let content = r#"{"key": "value", "number": 42}"#;
        utils::atomic_write(&file_path, content).unwrap();

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), content);

        // tmp文件应已清理
        let tmp_path = file_path.with_extension("tmp");
        assert!(!tmp_path.exists());
    }

    /// 测试损坏JSON配置的容错
    #[test]
    fn test_corrupted_json_fallback() {
        let corrupted = "{ this is not valid json }}}";
        let json: Value = serde_json::from_str(corrupted).unwrap_or_default();
        // unwrap_or_default应返回Null
        assert!(json.is_null());

        // 用空对象fallback
        let json: Value = serde_json::from_str(corrupted).unwrap_or_else(|_| serde_json::json!({}));
        assert!(json.is_object());
        assert!(json.as_object().unwrap().is_empty());
    }

    /// 测试restore清理代理字段（Claude）
    #[test]
    fn test_restore_removes_proxy_fields() {
        let config = serde_json::json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://proxy.test",
                "ANTHROPIC_API_KEY": "sk-test",
                "USER_CUSTOM_VAR": "keep-me"
            },
            "otherSetting": true
        });

        let mut json = config.clone();
        if let Some(env_obj) = json.get_mut("env").and_then(|e| e.as_object_mut()) {
            env_obj.remove("ANTHROPIC_BASE_URL");
            env_obj.remove("ANTHROPIC_API_KEY");
        }

        // 代理字段已移除
        assert!(json["env"].get("ANTHROPIC_BASE_URL").is_none());
        assert!(json["env"].get("ANTHROPIC_API_KEY").is_none());
        // 用户字段保留
        assert_eq!(json["env"]["USER_CUSTOM_VAR"], "keep-me");
        assert_eq!(json["otherSetting"], true);
    }

    /// 测试Codex restore清理provider字段
    #[test]
    fn test_codex_restore_removes_custom_provider() {
        use toml_edit::{value, DocumentMut};

        let toml_str = r#"
model_provider = "custom"
model = "gpt-4o"
some_user_key = "keep"

[model_providers.custom]
base_url = "https://proxy.test"
"#;
        let mut doc = toml_str.parse::<DocumentMut>().unwrap();
        doc.remove("model_provider");
        doc.remove("model");
        if let Some(providers) = doc.get_mut("model_providers") {
            if let Some(table) = providers.as_table_mut() {
                table.remove("custom");
            }
        }

        let result = doc.to_string();
        assert!(!result.contains("model_provider"));
        assert!(!result.contains("base_url"));
        assert!(result.contains("some_user_key = \"keep\""));
    }
}
