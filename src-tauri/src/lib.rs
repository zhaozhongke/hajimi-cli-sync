mod account;
mod auto_installer;
mod cli_sync;
mod droid_sync;
mod error;
mod extra_clients;
mod opencode_sync;
mod openclaw_sync;
mod system_check;
mod utils;

use cli_sync::CliApp;
use extra_clients::ExtraClient;
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

    // OpenClaw
    {
        let proxy_url = get_proxy_url("openclaw", &url);
        let (installed, version) = openclaw_sync::check_openclaw_installed();
        let (is_synced, has_backup, current_base_url) = if installed {
            openclaw_sync::get_sync_status(&proxy_url)
        } else {
            (false, false, None)
        };
        results.push(CliStatusResult {
            app: "openclaw".to_string(),
            installed,
            version,
            is_synced,
            has_backup,
            current_base_url,
            files: vec!["openclaw.json".to_string()],
            synced_count: None,
        });
    }

    // Extra clients (Chatbox, Cherry Studio, Jan, Cursor, Cline, Roo Code, Kilo Code, SillyTavern, LobeChat, BoltAI)
    for client in ExtraClient::all() {
        let proxy_url = get_proxy_url(client.as_str(), &url);
        let (installed, version) = extra_clients::check_extra_installed(client);
        let (is_synced, has_backup, current_base_url) = if installed {
            extra_clients::get_extra_sync_status(client, &proxy_url)
        } else {
            (false, false, None)
        };
        results.push(CliStatusResult {
            app: client.as_str().to_string(),
            installed,
            version,
            is_synced,
            has_backup,
            current_base_url,
            files: client.config_files_display(),
            synced_count: None,
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
            let cli_app = get_cli_app(&app).ok_or_else(|| format!("Invalid app: {}", app))?;
            cli_sync::sync_config(&cli_app, &proxy_url, &api_key, model.as_deref())
        }
        "opencode" => opencode_sync::sync_opencode_config(&proxy_url, &api_key).await,
        "openclaw" => openclaw_sync::sync_openclaw_config(&proxy_url, &api_key, model.as_deref()).await,
        "droid" => {
            droid_sync::sync_droid_config(&proxy_url, &api_key, model.as_deref()).map(|_| ())
        }
        other => {
            if let Some(client) = ExtraClient::from_str(other) {
                extra_clients::sync_extra_config(&client, &proxy_url, &api_key, model.as_deref())
            } else {
                Err(format!("Unknown app: {}", app))
            }
        }
    }
}

#[tauri::command]
async fn sync_all(
    url: String,
    api_key: String,
    model: Option<String>,
    per_cli_models: Option<std::collections::HashMap<String, String>>,
) -> Result<SyncAllResult, String> {
    // 检查系统环境
    system_check::validate_system_requirements().map_err(|e| e.to_string())?;

    utils::validate_url(&url).map_err(|e| e.to_string())?;
    if api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let cli_models = per_cli_models.unwrap_or_default();
    let apps = ["claude", "codex", "gemini", "opencode", "openclaw", "droid"];
    let mut results = Vec::new();

    for app_name in &apps {
        let proxy_url = get_proxy_url(app_name, &url);

        let installed = match *app_name {
            "claude" | "codex" | "gemini" => get_cli_app(app_name)
                .map(|app| cli_sync::check_cli_installed(&app).0)
                .unwrap_or(false),
            "opencode" => opencode_sync::check_opencode_installed().0,
            "openclaw" => openclaw_sync::check_openclaw_installed().0,
            "droid" => droid_sync::check_droid_installed().0,
            _ => false,
        };

        if !installed {
            continue;
        }

        // 优先使用per-cli model，fallback到全局default model
        let effective_model = cli_models
            .get(*app_name)
            .filter(|m| !m.is_empty())
            .or(model.as_ref());

        let result = match *app_name {
            "claude" | "codex" | "gemini" => match get_cli_app(app_name) {
                Some(cli_app) => cli_sync::sync_config(
                    &cli_app,
                    &proxy_url,
                    &api_key,
                    effective_model.map(|s| s.as_str()),
                ),
                None => Err(format!("Invalid app: {}", app_name)),
            },
            "opencode" => opencode_sync::sync_opencode_config(&proxy_url, &api_key).await,
            "openclaw" => openclaw_sync::sync_openclaw_config(&proxy_url, &api_key, effective_model.map(|s| s.as_str())).await,
            "droid" => droid_sync::sync_droid_config(
                &proxy_url,
                &api_key,
                effective_model.map(|s| s.as_str()),
            )
            .map(|_| ()),
            _ => continue,
        };

        results.push(SyncResult {
            app: app_name.to_string(),
            success: result.is_ok(),
            error: result.err(),
        });
    }

    // Extra clients
    for client in ExtraClient::all() {
        let app_name = client.as_str();
        let proxy_url = get_proxy_url(app_name, &url);
        let installed = extra_clients::check_extra_installed(client).0;

        if !installed {
            continue;
        }

        let effective_model = cli_models
            .get(app_name)
            .filter(|m| !m.is_empty())
            .or(model.as_ref());

        let result = extra_clients::sync_extra_config(
            client,
            &proxy_url,
            &api_key,
            effective_model.map(|s| s.as_str()),
        );

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
            let cli_app = get_cli_app(&app).ok_or_else(|| format!("Invalid app: {}", app))?;
            cli_sync::restore_config(&cli_app)
        }
        "opencode" => opencode_sync::restore_opencode_config(),
        "openclaw" => openclaw_sync::restore_openclaw_config(),
        "droid" => droid_sync::restore_droid_config(),
        other => {
            if let Some(client) = ExtraClient::from_str(other) {
                extra_clients::restore_extra_config(&client)
            } else {
                Err(format!("Unknown app: {}", app))
            }
        }
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
async fn test_connection(url: String, api_key: String) -> Result<String, String> {
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
                "Connection timed out (10s). Check the URL.".to_string()
            } else if e.is_connect() {
                format!("Cannot connect to server: {}", e)
            } else {
                format!("Request failed: {}", e)
            }
        })?;

    let status = response.status();
    if status.is_success() {
        Ok("ok".to_string())
    } else if status.as_u16() == 401 || status.as_u16() == 403 {
        Err("Invalid API key (401/403)".to_string())
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(format!("Server returned {}: {}", status, body))
    }
}

#[tauri::command]
async fn get_config_content(app: String, file_name: Option<String>) -> Result<String, String> {
    match app.as_str() {
        "claude" | "codex" | "gemini" => {
            let cli_app = get_cli_app(&app).ok_or_else(|| format!("Invalid app: {}", app))?;
            cli_sync::read_config_content(&cli_app, file_name.as_deref())
        }
        "opencode" => opencode_sync::read_opencode_config_content(),
        "openclaw" => openclaw_sync::read_openclaw_config_content(),
        "droid" => droid_sync::read_droid_config_content(),
        other => {
            if let Some(client) = ExtraClient::from_str(other) {
                extra_clients::read_extra_config_content(&client)
            } else {
                Err(format!("Unknown app: {}", app))
            }
        }
    }
}

#[tauri::command]
async fn write_config_file(app: String, file_name: String, content: String) -> Result<(), String> {
    match app.as_str() {
        "claude" | "codex" | "gemini" => {
            let cli_app = get_cli_app(&app).ok_or_else(|| format!("Invalid app: {}", app))?;
            cli_sync::write_config_content(&cli_app, &file_name, &content)
        }
        "opencode" => opencode_sync::write_opencode_config_content(&content),
        "openclaw" => openclaw_sync::write_openclaw_config_content(&content),
        "droid" => droid_sync::write_droid_config_content(&content),
        other => {
            if let Some(client) = ExtraClient::from_str(other) {
                extra_clients::write_extra_config_content(&client, &file_name, &content)
            } else {
                Err(format!("Unknown app: {}", other))
            }
        }
    }
}

#[tauri::command]
async fn open_external_url(url: String) -> Result<(), String> {
    open_path_in_system(&url)
}

#[tauri::command]
async fn launch_app(name: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-a", &name])
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", name, e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &name])
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", name, e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new(&name)
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", name, e))?;
    }
    Ok(())
}

#[tauri::command]
async fn open_config_folder(app: String) -> Result<(), String> {
    let folder = get_config_folder_path(&app)?;
    let folder_str = folder.to_string_lossy().to_string();
    open_path_in_system(&folder_str)
}

fn get_config_folder_path(app: &str) -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
    match app {
        "claude" => Ok(home.join(".claude")),
        "codex" => Ok(home.join(".codex")),
        "gemini" => Ok(home.join(".gemini")),
        "opencode" => {
            // XDG_CONFIG_HOME or ~/.config
            let config_dir = std::env::var("XDG_CONFIG_HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| home.join(".config"));
            Ok(config_dir.join("opencode"))
        }
        "openclaw" => Ok(home.join(".openclaw")),
        "droid" => {
            let config_dir = std::env::var("XDG_CONFIG_HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| home.join(".config"));
            Ok(config_dir.join("droid"))
        }
        other => {
            if let Some(client) = ExtraClient::from_str(other) {
                extra_clients::get_config_folder(&client)
                    .ok_or_else(|| format!("Cannot determine config folder for {}", other))
            } else {
                Err(format!("Unknown app: {}", other))
            }
        }
    }
}

fn open_path_in_system(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", path])
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    Ok(())
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
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(account::AccountState::new())
        .invoke_handler(tauri::generate_handler![
            get_all_cli_status,
            sync_cli,
            sync_all,
            restore_cli,
            get_config_content,
            write_config_file,
            fetch_models,
            test_connection,
            system_check::get_system_status,
            auto_installer::auto_install_dependencies,
            auto_installer::install_cli_tool,
            open_external_url,
            open_config_folder,
            launch_app,
            account::check_platform,
            account::account_login,
            account::account_get_tokens,
            account::account_check_session,
            account::account_restore_session,
            account::account_logout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
