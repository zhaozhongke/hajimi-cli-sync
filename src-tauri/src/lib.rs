mod auto_installer;
mod cli_sync;
mod droid_sync;
mod error;
mod opencode_sync;
mod system_check;
mod utils;

use cli_sync::CliApp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliStatusResult {
    pub app: String,
    pub installed: bool,
    pub version: Option<String>,
    pub is_synced: bool,
    pub has_backup: bool,
    pub current_base_url: Option<String>,
    pub files: Vec<String>,
    pub synced_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncAllResult {
    pub results: Vec<SyncResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncResult {
    pub app: String,
    pub success: bool,
    pub error: Option<String>,
}

fn get_cli_app(app: &str) -> Option<CliApp> {
    match app {
        "claude" => Some(CliApp::Claude),
        "codex" => Some(CliApp::Codex),
        "gemini" => Some(CliApp::Gemini),
        _ => None,
    }
}

/// Get the appropriate proxy URL for each CLI tool
fn get_proxy_url(app: &str, base_url: &str) -> String {
    let url = base_url.trim_end_matches('/');
    match app {
        "codex" | "opencode" => {
            if url.ends_with("/v1") {
                url.to_string()
            } else {
                format!("{}/v1", url)
            }
        }
        _ => url.to_string(),
    }
}

#[tauri::command]
async fn get_all_cli_status(url: String) -> Result<Vec<CliStatusResult>, String> {
    // 🚀 自动安装缺失依赖（后台静默）
    tokio::spawn(async {
        if let Err(e) = auto_installer::auto_install_dependencies().await {
            tracing::warn!("[auto_install] Failed to auto-install dependencies: {:?}", e);
        }
    });

    // 首先检查系统环境
    if let Err(e) = system_check::validate_system_requirements() {
        tracing::warn!("[get_all_cli_status] System check warning: {}", e);
    }

    if let Err(e) = utils::validate_url(&url) {
        return Err(e.to_string());
    }

    let mut results = Vec::new();

    for app_name in &["claude", "codex", "gemini"] {
        if let Some(app) = get_cli_app(app_name) {
            let proxy_url = get_proxy_url(app_name, &url);
            let (installed, version) = cli_sync::check_cli_installed(&app);
            let (is_synced, has_backup, current_base_url) = if installed {
                cli_sync::get_sync_status(&app, &proxy_url)
            } else {
                (false, false, None)
            };
            results.push(CliStatusResult {
                app: app_name.to_string(),
                installed,
                version,
                is_synced,
                has_backup,
                current_base_url,
                files: app.config_files().into_iter().map(|f| f.name).collect(),
                synced_count: None,
            });
        }
    }

    // OpenCode
    {
        let proxy_url = get_proxy_url("opencode", &url);
        let (installed, version) = opencode_sync::check_opencode_installed();
        let (is_synced, has_backup, current_base_url) = if installed {
            opencode_sync::get_sync_status(&proxy_url)
        } else {
            (false, false, None)
        };
        results.push(CliStatusResult {
            app: "opencode".to_string(),
            installed,
            version,
            is_synced,
            has_backup,
            current_base_url,
            files: vec!["opencode.json".to_string()],
            synced_count: None,
        });
    }

    // Droid
    {
        let proxy_url = get_proxy_url("droid", &url);
        let (installed, version) = droid_sync::check_droid_installed();
        let (is_synced, has_backup, current_base_url, synced_count) = if installed {
            droid_sync::get_sync_status(&proxy_url)
        } else {
            (false, false, None, 0)
        };
        results.push(CliStatusResult {
            app: "droid".to_string(),
            installed,
            version,
            is_synced,
            has_backup,
            current_base_url,
            files: vec!["settings.json".to_string()],
            synced_count: Some(synced_count),
        });
    }

    Ok(results)
}

#[tauri::command]
async fn sync_cli(
    app: String,
    url: String,
    api_key: String,
    model: Option<String>,
) -> Result<(), String> {
    // 检查系统环境
    system_check::validate_system_requirements().map_err(|e| e.to_string())?;

    utils::validate_url(&url).map_err(|e| e.to_string())?;
    if api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let proxy_url = get_proxy_url(&app, &url);

    match app.as_str() {
        "claude" | "codex" | "gemini" => {
            let cli_app = get_cli_app(&app)
                .ok_or_else(|| format!("Invalid app: {}", app))?;
            cli_sync::sync_config(&cli_app, &proxy_url, &api_key, model.as_deref())
        }
        "opencode" => opencode_sync::sync_opencode_config(&proxy_url, &api_key),
        "droid" => {
            droid_sync::sync_droid_config(&proxy_url, &api_key, model.as_deref())
                .map(|_| ())
        }
        _ => Err(format!("Unknown app: {}", app)),
    }
}

#[tauri::command]
async fn sync_all(
    url: String,
    api_key: String,
    model: Option<String>,
) -> Result<SyncAllResult, String> {
    // 检查系统环境
    system_check::validate_system_requirements().map_err(|e| e.to_string())?;

    utils::validate_url(&url).map_err(|e| e.to_string())?;
    if api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let apps = ["claude", "codex", "gemini", "opencode", "droid"];
    let mut results = Vec::new();

    for app_name in &apps {
        let proxy_url = get_proxy_url(app_name, &url);

        let installed = match *app_name {
            "claude" | "codex" | "gemini" => {
                get_cli_app(app_name)
                    .map(|app| cli_sync::check_cli_installed(&app).0)
                    .unwrap_or(false)
            }
            "opencode" => opencode_sync::check_opencode_installed().0,
            "droid" => droid_sync::check_droid_installed().0,
            _ => false,
        };

        if !installed {
            continue;
        }

        let result = match *app_name {
            "claude" | "codex" | "gemini" => {
                match get_cli_app(app_name) {
                    Some(cli_app) => cli_sync::sync_config(
                        &cli_app, &proxy_url, &api_key, model.as_deref(),
                    ),
                    None => Err(format!("Invalid app: {}", app_name)),
                }
            }
            "opencode" => opencode_sync::sync_opencode_config(&proxy_url, &api_key),
            "droid" => droid_sync::sync_droid_config(
                &proxy_url, &api_key, model.as_deref(),
            ).map(|_| ()),
            _ => continue,
        };

        results.push(SyncResult {
            app: app_name.to_string(),
            success: result.is_ok(),
            error: result.err(),
        });
    }

    Ok(SyncAllResult { results })
}

#[tauri::command]
async fn restore_cli(app: String) -> Result<(), String> {
    match app.as_str() {
        "claude" | "codex" | "gemini" => {
            let cli_app = get_cli_app(&app)
                .ok_or_else(|| format!("Invalid app: {}", app))?;
            cli_sync::restore_config(&cli_app)
        }
        "opencode" => opencode_sync::restore_opencode_config(),
        "droid" => droid_sync::restore_droid_config(),
        _ => Err(format!("Unknown app: {}", app)),
    }
}

#[tauri::command]
async fn fetch_models(url: String, api_key: String) -> Result<Vec<String>, String> {
    utils::validate_url(&url).map_err(|e| e.to_string())?;
    if api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let models_url = format!("{}/v1/models", url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&models_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "Request timed out (10s)".to_string()
            } else if e.is_connect() {
                format!("Connection failed: {}", e)
            } else {
                format!("Request failed: {}", e)
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API returned {}: {}", status, body));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let mut models: Vec<String> = Vec::new();

    if let Some(data) = body.get("data").and_then(|v| v.as_array()) {
        for item in data {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                models.push(id.to_string());
            }
        }
    }

    models.sort();
    Ok(models)
}

#[tauri::command]
async fn get_config_content(app: String, file_name: Option<String>) -> Result<String, String> {
    match app.as_str() {
        "claude" | "codex" | "gemini" => {
            let cli_app = get_cli_app(&app)
                .ok_or_else(|| format!("Invalid app: {}", app))?;
            cli_sync::read_config_content(&cli_app, file_name.as_deref())
        }
        "opencode" => opencode_sync::read_opencode_config_content(),
        "droid" => droid_sync::read_droid_config_content(),
        _ => Err(format!("Unknown app: {}", app)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_all_cli_status,
            sync_cli,
            sync_all,
            restore_cli,
            get_config_content,
            fetch_models,
            system_check::get_system_status,
            auto_installer::auto_install_dependencies,
            auto_installer::install_cli_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
