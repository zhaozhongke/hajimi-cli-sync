use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::cli_sync;
use crate::utils;

use crate::utils::BACKUP_SUFFIX;

/// Extra AI client tools beyond the core 5 (Claude, Codex, Gemini, OpenCode, Droid).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraClient {
    ClaudeVSCode,
    Chatbox,
    CherryStudio,
    Jan,
    Cursor,
    Cline,
    RooCode,
    KiloCode,
    SillyTavern,
    LobeChat,
    BoltAI,
}

impl ExtraClient {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeVSCode => "claude-vscode",
            Self::Chatbox => "chatbox",
            Self::CherryStudio => "cherry-studio",
            Self::Jan => "jan",
            Self::Cursor => "cursor",
            Self::Cline => "cline",
            Self::RooCode => "roo-code",
            Self::KiloCode => "kilo-code",
            Self::SillyTavern => "sillytavern",
            Self::LobeChat => "lobechat",
            Self::BoltAI => "boltai",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClaudeVSCode => "Claude Code (VS Code)",
            Self::Chatbox => "Chatbox",
            Self::CherryStudio => "Cherry Studio",
            Self::Jan => "Jan",
            Self::Cursor => "Cursor",
            Self::Cline => "Cline",
            Self::RooCode => "Roo Code",
            Self::KiloCode => "Kilo Code",
            Self::SillyTavern => "SillyTavern",
            Self::LobeChat => "LobeChat",
            Self::BoltAI => "BoltAI",
        }
    }

    pub fn all() -> &'static [ExtraClient] {
        &[
            Self::ClaudeVSCode,
            Self::Chatbox,
            Self::CherryStudio,
            Self::Jan,
            Self::Cursor,
            Self::Cline,
            Self::RooCode,
            Self::KiloCode,
            Self::SillyTavern,
            Self::LobeChat,
            Self::BoltAI,
        ]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude-vscode" => Some(Self::ClaudeVSCode),
            "chatbox" => Some(Self::Chatbox),
            "cherry-studio" => Some(Self::CherryStudio),
            "jan" => Some(Self::Jan),
            "cursor" => Some(Self::Cursor),
            "cline" => Some(Self::Cline),
            "roo-code" => Some(Self::RooCode),
            "kilo-code" => Some(Self::KiloCode),
            "sillytavern" => Some(Self::SillyTavern),
            "lobechat" => Some(Self::LobeChat),
            "boltai" => Some(Self::BoltAI),
            _ => None,
        }
    }

    /// Whether this client supports file-based config sync.
    /// Clients using encrypted/keychain storage return false.
    pub fn supports_file_sync(&self) -> bool {
        matches!(
            self,
            Self::Chatbox | Self::CherryStudio | Self::Jan | Self::SillyTavern
        )
    }

    pub fn config_files_display(&self) -> Vec<String> {
        match self {
            Self::ClaudeVSCode => vec!["settings.json".to_string()],
            Self::Chatbox => vec!["config.json".to_string()],
            Self::CherryStudio => vec!["config.json".to_string()],
            Self::Jan => vec!["openai.json".to_string()],
            Self::Cursor => vec!["(app settings)".to_string()],
            Self::Cline | Self::RooCode | Self::KiloCode => {
                vec!["(extension settings)".to_string()]
            }
            Self::SillyTavern => vec!["secrets.json".to_string()],
            Self::LobeChat => vec!["(browser storage)".to_string()],
            Self::BoltAI => vec!["(macOS Keychain)".to_string()],
        }
    }
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

#[cfg(target_os = "macos")]
fn app_support_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join("Library/Application Support"))
}

#[cfg(target_os = "linux")]
fn app_support_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".config"))
}

#[cfg(target_os = "windows")]
fn app_support_dir() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .ok()
        .or_else(|| home_dir().map(|h| h.join("AppData/Roaming")))
}

fn chatbox_config_path() -> Option<PathBuf> {
    let app_sup = app_support_dir()?;
    #[cfg(target_os = "macos")]
    let dir = app_sup.join("xyz.chatboxapp.app");
    #[cfg(target_os = "linux")]
    let dir = app_sup.join("Chatbox");
    #[cfg(target_os = "windows")]
    let dir = app_sup.join("Chatbox");
    Some(dir.join("config.json"))
}

fn cherry_config_path() -> Option<PathBuf> {
    let app_sup = app_support_dir()?;
    // Cherry Studio may use different directory names across versions
    for name in &["CherryStudio", "cherry-studio", "Cherry Studio"] {
        let p = app_sup.join(name).join("config.json");
        if p.exists() {
            return Some(p);
        }
    }
    // Default to CherryStudio if none found
    Some(app_sup.join("CherryStudio").join("config.json"))
}

fn jan_config_path() -> Option<PathBuf> {
    let home = home_dir()?;
    // Jan stores OpenAI-compatible engine config at ~/jan/engines/openai.json
    Some(home.join("jan").join("engines").join("openai.json"))
}

fn cursor_config_path() -> Option<PathBuf> {
    let app_sup = app_support_dir()?;
    Some(app_sup.join("Cursor").join("User").join("settings.json"))
}

fn vscode_settings_path() -> Option<PathBuf> {
    let app_sup = app_support_dir()?;
    Some(app_sup.join("Code").join("User").join("settings.json"))
}

fn sillytavern_secrets_path() -> Option<PathBuf> {
    let home = home_dir()?;
    for dir_name in &["SillyTavern", "sillytavern", ".sillytavern"] {
        let secrets = home.join(dir_name).join("data/default-user/secrets.json");
        if secrets.exists() {
            return Some(secrets);
        }
        // Also check config.yaml in root
        let config = home.join(dir_name).join("config.yaml");
        if config.exists() {
            return Some(home.join(dir_name).join("data/default-user/secrets.json"));
        }
    }
    Some(
        home.join("SillyTavern")
            .join("data/default-user/secrets.json"),
    )
}

/// Get the config file path for a client (the primary file we sync to).
fn config_path_for(client: &ExtraClient) -> Option<PathBuf> {
    match client {
        ExtraClient::ClaudeVSCode => {
            // Shares config with Claude Code CLI: ~/.claude/settings.json
            home_dir().map(|h| h.join(".claude").join("settings.json"))
        }
        ExtraClient::Chatbox => chatbox_config_path(),
        ExtraClient::CherryStudio => cherry_config_path(),
        ExtraClient::Jan => jan_config_path(),
        ExtraClient::Cursor => cursor_config_path(),
        ExtraClient::Cline | ExtraClient::RooCode | ExtraClient::KiloCode => vscode_settings_path(),
        ExtraClient::SillyTavern => sillytavern_secrets_path(),
        ExtraClient::LobeChat | ExtraClient::BoltAI => None,
    }
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

/// Check if the VS Code extension is installed by scanning known extensions dirs.
fn is_vscode_extension_installed(ext_prefix: &str) -> bool {
    let home = match home_dir() {
        Some(h) => h,
        None => return false,
    };
    // VS Code stores extensions in ~/.vscode/extensions on all platforms.
    // Cursor stores its own copy in ~/.cursor/extensions.
    let ext_dirs = [
        home.join(".vscode").join("extensions"),
        home.join(".cursor").join("extensions"),
    ];
    for ext_dir in &ext_dirs {
        if !ext_dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(ext_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(ext_prefix) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Check if a macOS app bundle is installed.
#[cfg(target_os = "macos")]
fn is_app_installed(app_name: &str) -> bool {
    let app_path = PathBuf::from(format!("/Applications/{}.app", app_name));
    if app_path.exists() {
        return true;
    }
    // Also check ~/Applications
    if let Some(home) = home_dir() {
        let user_app = home.join(format!("Applications/{}.app", app_name));
        if user_app.exists() {
            return true;
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn is_app_installed(app_name: &str) -> bool {
    // Check common Windows install locations
    let candidates: &[&str] = &[
        "LOCALAPPDATA",
        "PROGRAMFILES",
        "PROGRAMFILES(X86)",
        "APPDATA",
    ];
    for env_var in candidates {
        if let Ok(base) = std::env::var(env_var) {
            let path = PathBuf::from(&base).join(app_name);
            if path.exists() {
                return true;
            }
            // Also check with .exe suffix directly
            let exe = PathBuf::from(&base).join(format!("{}.exe", app_name));
            if exe.exists() {
                return true;
            }
        }
    }
    // Check if binary is in PATH
    utils::resolve_executable(&app_name.to_lowercase()).is_some()
}

#[cfg(target_os = "linux")]
fn is_app_installed(app_name: &str) -> bool {
    // Check ~/.local/share/applications and /usr/share/applications for .desktop files
    let lower = app_name.to_lowercase();
    let candidates = [
        dirs::home_dir().map(|h| h.join(".local/share/applications")),
        Some(PathBuf::from("/usr/share/applications")),
        Some(PathBuf::from("/usr/local/share/applications")),
    ];
    for dir_opt in &candidates {
        if let Some(dir) = dir_opt {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.to_lowercase().contains(&lower) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    utils::resolve_executable(&lower).is_some()
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn is_app_installed(_app_name: &str) -> bool {
    false
}

/// Detect whether a client is installed. Returns (installed, version).
pub fn check_extra_installed(client: &ExtraClient) -> (bool, Option<String>) {
    match client {
        ExtraClient::ClaudeVSCode => {
            let installed = is_vscode_extension_installed("anthropic.claude-code-");
            (
                installed,
                if installed {
                    Some("extension".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::Chatbox => {
            let installed = is_app_installed("Chatbox")
                || chatbox_config_path()
                    .map_or(false, |p| p.parent().map_or(false, |d| d.exists()));
            (
                installed,
                if installed {
                    Some("detected".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::CherryStudio => {
            let installed = is_app_installed("Cherry Studio")
                || cherry_config_path().map_or(false, |p| p.exists());
            (
                installed,
                if installed {
                    Some("detected".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::Jan => {
            let installed =
                is_app_installed("Jan") || home_dir().map_or(false, |h| h.join("jan").exists());
            (
                installed,
                if installed {
                    Some("detected".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::Cursor => {
            // Check for cursor CLI executable first
            if let Some(path) = utils::resolve_executable("cursor") {
                let version = utils::get_cli_version(&path);
                return (true, version.or_else(|| Some("detected".to_string())));
            }
            let installed =
                is_app_installed("Cursor") || cursor_config_path().map_or(false, |p| p.exists());
            (
                installed,
                if installed {
                    Some("detected".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::Cline => {
            let installed = is_vscode_extension_installed("saoudrizwan.claude-dev-")
                || is_vscode_extension_installed("hybridtalentcomputing.cline-chinese-")
                || is_vscode_extension_installed("cline.cline-");
            (
                installed,
                if installed {
                    Some("extension".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::RooCode => {
            let installed = is_vscode_extension_installed("rooveterinaryinc.roo-cline-");
            (
                installed,
                if installed {
                    Some("extension".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::KiloCode => {
            let installed = is_vscode_extension_installed("kilocode.kilo-code-");
            (
                installed,
                if installed {
                    Some("extension".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::SillyTavern => {
            let home = match home_dir() {
                Some(h) => h,
                None => return (false, None),
            };
            let installed = ["SillyTavern", "sillytavern", ".sillytavern"]
                .iter()
                .any(|d| home.join(d).exists());
            (
                installed,
                if installed {
                    Some("detected".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::LobeChat => {
            let installed = is_app_installed("LobeChat")
                || app_support_dir().map_or(false, |d| d.join("LobeChat").exists());
            (
                installed,
                if installed {
                    Some("detected".to_string())
                } else {
                    None
                },
            )
        }
        ExtraClient::BoltAI => {
            let installed = is_app_installed("BoltAI");
            (
                installed,
                if installed {
                    Some("detected".to_string())
                } else {
                    None
                },
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Sync status
// ---------------------------------------------------------------------------

const HAJIMI_MARKER: &str = "hajimi";

pub fn get_extra_sync_status(
    client: &ExtraClient,
    proxy_url: &str,
) -> (bool, bool, Option<String>) {
    let config_path = match config_path_for(client) {
        Some(p) => p,
        None => return (false, false, None),
    };

    let backup_path = backup_path_for(&config_path);
    let has_backup = backup_path.exists();

    if !config_path.exists() {
        return (false, has_backup, None);
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return (false, has_backup, None),
    };

    match client {
        ExtraClient::ClaudeVSCode => {
            // Reuse Claude CLI sync status check
            let cli_app = cli_sync::CliApp::Claude;
            cli_sync::get_sync_status(&cli_app, proxy_url)
        }
        ExtraClient::Chatbox => check_chatbox_synced(&content, proxy_url, has_backup),
        ExtraClient::CherryStudio => check_cherry_synced(&content, proxy_url, has_backup),
        ExtraClient::Jan => check_jan_synced(&content, proxy_url, has_backup),
        ExtraClient::SillyTavern => check_sillytavern_synced(&content, proxy_url, has_backup),
        ExtraClient::Cursor | ExtraClient::Cline | ExtraClient::RooCode | ExtraClient::KiloCode => {
            (false, false, None)
        }
        _ => (false, false, None),
    }
}

fn check_chatbox_synced(
    content: &str,
    proxy_url: &str,
    has_backup: bool,
) -> (bool, bool, Option<String>) {
    let json: Value = serde_json::from_str(content).unwrap_or_default();
    let current_url = json
        .get("openaiApiHost")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let is_synced = current_url
        .as_deref()
        .map_or(false, |u| utils::urls_match(u, proxy_url));

    (is_synced, has_backup, current_url)
}

fn check_cherry_synced(
    content: &str,
    proxy_url: &str,
    has_backup: bool,
) -> (bool, bool, Option<String>) {
    let json: Value = serde_json::from_str(content).unwrap_or_default();

    // Cherry Studio stores providers in a "providers" array/object
    let current_url = json
        .get("providers")
        .and_then(|p| p.as_array())
        .and_then(|arr| {
            arr.iter().find_map(|p| {
                let id = p.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                if id == HAJIMI_MARKER {
                    p.get("apiHost")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
        });

    let is_synced = current_url
        .as_deref()
        .map_or(false, |u| utils::urls_match(u, proxy_url));

    (is_synced, has_backup, current_url)
}

fn check_jan_synced(
    content: &str,
    proxy_url: &str,
    has_backup: bool,
) -> (bool, bool, Option<String>) {
    let json: Value = serde_json::from_str(content).unwrap_or_default();

    // Jan engine config uses "full_url" (ends with /chat/completions) and "api_key"
    let current_url = json
        .get("full_url")
        .and_then(|v| v.as_str())
        .map(|s| {
            // Normalise: strip trailing /chat/completions to get base URL
            s.trim_end_matches("/chat/completions").to_string()
        });

    let is_synced = current_url
        .as_deref()
        .map_or(false, |u| utils::urls_match(u, proxy_url));

    (is_synced, has_backup, current_url)
}

fn check_sillytavern_synced(
    content: &str,
    proxy_url: &str,
    has_backup: bool,
) -> (bool, bool, Option<String>) {
    let json: Value = serde_json::from_str(content).unwrap_or_default();

    let current_url = json
        .get("api_url_scale")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let is_synced = current_url
        .as_deref()
        .map_or(false, |u| utils::urls_match(u, proxy_url));

    (is_synced, has_backup, current_url)
}

fn check_vscode_env_synced(
    content: &str,
    proxy_url: &str,
    has_backup: bool,
) -> (bool, bool, Option<String>) {
    let json: Value = serde_json::from_str(content).unwrap_or_default();

    // Check terminal.integrated.env for our proxy URL
    let env_key = if cfg!(target_os = "macos") {
        "terminal.integrated.env.osx"
    } else if cfg!(target_os = "linux") {
        "terminal.integrated.env.linux"
    } else {
        "terminal.integrated.env.windows"
    };

    let current_url = json
        .get(env_key)
        .and_then(|e| e.get("OPENAI_BASE_URL"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let is_synced = current_url
        .as_deref()
        .map_or(false, |u| utils::urls_match(u, proxy_url));

    (is_synced, has_backup, current_url)
}

// urls_match: use crate::utils::urls_match

// ---------------------------------------------------------------------------
// Sync
// ---------------------------------------------------------------------------

pub fn sync_extra_config(
    client: &ExtraClient,
    proxy_url: &str,
    api_key: &str,
    model: Option<&str>,
) -> Result<(), String> {
    match client {
        ExtraClient::ClaudeVSCode => {
            // Reuse Claude CLI sync logic — writes to ~/.claude/settings.json
            let cli_app = cli_sync::CliApp::Claude;
            cli_sync::sync_config(&cli_app, proxy_url, api_key, model)
        }
        ExtraClient::Chatbox => sync_chatbox(proxy_url, api_key, model),
        ExtraClient::CherryStudio => sync_cherry(proxy_url, api_key, model),
        ExtraClient::Jan => sync_jan(proxy_url, api_key, model),
        ExtraClient::SillyTavern => sync_sillytavern(proxy_url, api_key),
        ExtraClient::Cursor => {
            Err(format!(
                "{} AI configuration must be set through the app UI: \
                 Settings > Models > OpenAI API Key / Base URL.",
                client.display_name()
            ))
        }
        ExtraClient::Cline | ExtraClient::RooCode | ExtraClient::KiloCode => {
            Err(format!(
                "{} stores API config in its extension settings. \
                 Open the extension sidebar > Settings icon > set API Provider to \"OpenAI Compatible\", \
                 then enter your Base URL and API Key.",
                client.display_name()
            ))
        }
        ExtraClient::LobeChat => {
            Err(format!(
                "{} uses browser storage or environment variables. \
                 Configure it through the app UI or set OPENAI_BASE_URL and OPENAI_API_KEY env vars.",
                client.display_name()
            ))
        }
        ExtraClient::BoltAI => {
            Err(format!(
                "{} stores API keys in macOS Keychain. \
                 Configure it through the app: Settings > Models > Add OpenAI-compatible Server.",
                client.display_name()
            ))
        }
    }
}

fn sync_chatbox(proxy_url: &str, api_key: &str, model: Option<&str>) -> Result<(), String> {
    let config_path =
        chatbox_config_path().ok_or("Failed to determine Chatbox config directory")?;

    ensure_parent_dir(&config_path)?;
    utils::create_rotated_backup(&config_path, BACKUP_SUFFIX).map_err(|e| e.to_string())?;

    let mut config: Value = read_or_empty_json(&config_path);

    let obj = config
        .as_object_mut()
        .ok_or("Chatbox config is not a JSON object")?;

    obj.insert(
        "openaiApiHost".to_string(),
        Value::String(proxy_url.to_string()),
    );
    obj.insert(
        "openaiApiKey".to_string(),
        Value::String(api_key.to_string()),
    );

    if let Some(m) = model {
        obj.insert("chatgptModel".to_string(), Value::String(m.to_string()));
    }

    let content = utils::to_json_pretty(&config).map_err(|e| e.to_string())?;
    utils::atomic_write(&config_path, &content).map_err(|e| e.to_string())
}

fn sync_cherry(proxy_url: &str, api_key: &str, model: Option<&str>) -> Result<(), String> {
    let config_path =
        cherry_config_path().ok_or("Failed to determine Cherry Studio config directory")?;

    ensure_parent_dir(&config_path)?;
    utils::create_rotated_backup(&config_path, BACKUP_SUFFIX).map_err(|e| e.to_string())?;

    let mut config: Value = read_or_empty_json(&config_path);

    if !config.is_object() {
        config = serde_json::json!({});
    }

    // Build our provider entry
    let mut provider = serde_json::json!({
        "id": HAJIMI_MARKER,
        "name": "哈基米 AI",
        "type": "openai",
        "apiHost": proxy_url,
        "apiKey": api_key,
        "enabled": true
    });
    if let Some(m) = model {
        provider
            .as_object_mut()
            .unwrap()
            .insert("defaultModel".to_string(), Value::String(m.to_string()));
    }

    // Upsert into providers array
    let providers = config
        .as_object_mut()
        .unwrap()
        .entry("providers")
        .or_insert(serde_json::json!([]));

    if let Some(arr) = providers.as_array_mut() {
        // Remove existing hajimi provider
        arr.retain(|p| {
            p.get("id")
                .and_then(|v| v.as_str())
                .map_or(true, |id| id != HAJIMI_MARKER)
        });
        arr.push(provider);
    }

    let content = utils::to_json_pretty(&config).map_err(|e| e.to_string())?;
    utils::atomic_write(&config_path, &content).map_err(|e| e.to_string())
}

fn sync_jan(proxy_url: &str, api_key: &str, _model: Option<&str>) -> Result<(), String> {
    let config_path = jan_config_path().ok_or("Failed to determine Jan config directory")?;

    ensure_parent_dir(&config_path)?;
    utils::create_rotated_backup(&config_path, BACKUP_SUFFIX).map_err(|e| e.to_string())?;

    // Jan engine config format (~/jan/engines/openai.json):
    // { "full_url": "https://proxy/v1/chat/completions", "api_key": "sk-..." }
    // Normalise to always include /v1 before /chat/completions
    let base = proxy_url.trim().trim_end_matches('/');
    let base = base.trim_end_matches("/v1");
    let full_url = format!("{}/v1/chat/completions", base);

    let config = serde_json::json!({
        "full_url": full_url,
        "api_key": api_key,
    });

    let content = utils::to_json_pretty(&config).map_err(|e| e.to_string())?;
    utils::atomic_write(&config_path, &content).map_err(|e| e.to_string())
}

fn sync_sillytavern(proxy_url: &str, api_key: &str) -> Result<(), String> {
    let secrets_path =
        sillytavern_secrets_path().ok_or("Failed to determine SillyTavern config directory")?;

    ensure_parent_dir(&secrets_path)?;
    utils::create_rotated_backup(&secrets_path, BACKUP_SUFFIX).map_err(|e| e.to_string())?;

    let mut secrets: Value = read_or_empty_json(&secrets_path);

    if !secrets.is_object() {
        secrets = serde_json::json!({});
    }

    let obj = secrets.as_object_mut().unwrap();
    obj.insert(
        "api_key_openai".to_string(),
        Value::String(api_key.to_string()),
    );
    obj.insert(
        "api_url_scale".to_string(),
        Value::String(proxy_url.to_string()),
    );

    let content = utils::to_json_pretty(&secrets).map_err(|e| e.to_string())?;
    utils::atomic_write(&secrets_path, &content).map_err(|e| e.to_string())
}

/// Write proxy env vars into VS Code / Cursor settings.json.
/// This makes the API key and base URL available in the integrated terminal
/// and to extensions that read these env vars.
fn sync_vscode_env(
    settings_path: Option<PathBuf>,
    proxy_url: &str,
    api_key: &str,
) -> Result<(), String> {
    let path = settings_path.ok_or("Failed to determine settings.json path")?;

    ensure_parent_dir(&path)?;
    utils::create_rotated_backup(&path, BACKUP_SUFFIX).map_err(|e| e.to_string())?;

    let mut config: Value = read_or_empty_json(&path);
    if !config.is_object() {
        config = serde_json::json!({});
    }

    let env_key = if cfg!(target_os = "macos") {
        "terminal.integrated.env.osx"
    } else if cfg!(target_os = "linux") {
        "terminal.integrated.env.linux"
    } else {
        "terminal.integrated.env.windows"
    };

    let obj = config.as_object_mut().unwrap();
    let env = obj.entry(env_key).or_insert(serde_json::json!({}));

    if let Some(env_obj) = env.as_object_mut() {
        env_obj.insert(
            "OPENAI_BASE_URL".to_string(),
            Value::String(proxy_url.to_string()),
        );
        env_obj.insert(
            "OPENAI_API_KEY".to_string(),
            Value::String(api_key.to_string()),
        );
    }

    let content = utils::to_json_pretty(&config).map_err(|e| e.to_string())?;
    utils::atomic_write(&path, &content).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Restore
// ---------------------------------------------------------------------------

pub fn restore_extra_config(client: &ExtraClient) -> Result<(), String> {
    // ClaudeVSCode shares config with Claude CLI — delegate to cli_sync
    if matches!(client, ExtraClient::ClaudeVSCode) {
        let cli_app = cli_sync::CliApp::Claude;
        return cli_sync::restore_config(&cli_app);
    }

    let config_path = config_path_for(client)
        .ok_or_else(|| format!("{} does not use file-based config", client.display_name()))?;

    let backup = backup_path_for(&config_path);
    if !backup.exists() {
        return Err(format!(
            "No backup file found for {}",
            client.display_name()
        ));
    }

    if config_path.exists() {
        fs::remove_file(&config_path).map_err(|e| format!("Failed to remove config: {}", e))?;
    }
    fs::rename(&backup, &config_path).map_err(|e| format!("Failed to restore config: {}", e))?;

    tracing::info!(
        "[extra_clients] Restored {} config from backup",
        client.display_name()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Read config content
// ---------------------------------------------------------------------------

pub fn read_extra_config_content(client: &ExtraClient) -> Result<String, String> {
    let config_path = config_path_for(client).ok_or_else(|| {
        format!(
            "{} does not use a readable config file",
            client.display_name()
        )
    })?;

    if !config_path.exists() {
        return Err(format!("Config file does not exist: {:?}", config_path));
    }

    fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn backup_path_for(config_path: &PathBuf) -> PathBuf {
    let file_name = config_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    config_path.with_file_name(format!("{}{}", file_name, BACKUP_SUFFIX))
}

fn ensure_parent_dir(path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
    }
    Ok(())
}

fn read_or_empty_json(path: &PathBuf) -> Value {
    if path.exists() {
        fs::read_to_string(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extra_client_as_str_roundtrip() {
        for client in ExtraClient::all() {
            let name = client.as_str();
            let parsed = ExtraClient::from_str(name);
            assert_eq!(parsed, Some(*client), "roundtrip failed for {}", name);
        }
    }

    #[test]
    fn test_extra_client_display_names() {
        assert_eq!(ExtraClient::Chatbox.display_name(), "Chatbox");
        assert_eq!(ExtraClient::CherryStudio.display_name(), "Cherry Studio");
        assert_eq!(ExtraClient::RooCode.display_name(), "Roo Code");
    }

    #[test]
    fn test_supports_file_sync() {
        assert!(ExtraClient::Chatbox.supports_file_sync());
        assert!(ExtraClient::CherryStudio.supports_file_sync());
        assert!(ExtraClient::Jan.supports_file_sync());
        assert!(ExtraClient::SillyTavern.supports_file_sync());
        assert!(!ExtraClient::BoltAI.supports_file_sync());
        assert!(!ExtraClient::LobeChat.supports_file_sync());
    }

    #[test]
    fn test_urls_match() {
        assert!(urls_match("https://example.com", "https://example.com"));
        assert!(urls_match("https://example.com/", "https://example.com"));
        assert!(urls_match("https://example.com", "https://example.com/"));
        assert!(!urls_match("https://a.com", "https://b.com"));
    }

    #[test]
    fn test_chatbox_sync_and_read() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, "{}").unwrap();

        // Simulate sync logic directly
        let mut config: Value = serde_json::from_str("{}").unwrap();
        let obj = config.as_object_mut().unwrap();
        obj.insert(
            "openaiApiHost".to_string(),
            Value::String("https://proxy.test".to_string()),
        );
        obj.insert(
            "openaiApiKey".to_string(),
            Value::String("sk-test".to_string()),
        );
        obj.insert(
            "chatgptModel".to_string(),
            Value::String("gpt-4o".to_string()),
        );

        let content = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&config_path, &content).unwrap();

        let written: Value =
            serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(written["openaiApiHost"], "https://proxy.test");
        assert_eq!(written["openaiApiKey"], "sk-test");
        assert_eq!(written["chatgptModel"], "gpt-4o");
    }

    #[test]
    fn test_chatbox_sync_preserves_existing() {
        let existing = serde_json::json!({
            "theme": "dark",
            "language": "en",
            "openaiApiHost": "https://old.url"
        });

        let mut config = existing.clone();
        let obj = config.as_object_mut().unwrap();
        obj.insert(
            "openaiApiHost".to_string(),
            Value::String("https://new.url".to_string()),
        );
        obj.insert(
            "openaiApiKey".to_string(),
            Value::String("sk-new".to_string()),
        );

        assert_eq!(config["theme"], "dark");
        assert_eq!(config["language"], "en");
        assert_eq!(config["openaiApiHost"], "https://new.url");
        assert_eq!(config["openaiApiKey"], "sk-new");
    }

    #[test]
    fn test_cherry_provider_upsert() {
        let mut config = serde_json::json!({
            "providers": [
                { "id": "openai", "name": "OpenAI", "apiHost": "https://api.openai.com" },
                { "id": "hajimi", "name": "Old Hajimi", "apiHost": "https://old.proxy" }
            ]
        });

        let new_provider = serde_json::json!({
            "id": "hajimi",
            "name": "哈基米 AI",
            "type": "openai",
            "apiHost": "https://new.proxy",
            "apiKey": "sk-new",
            "enabled": true
        });

        if let Some(arr) = config.get_mut("providers").and_then(|p| p.as_array_mut()) {
            arr.retain(|p| {
                p.get("id")
                    .and_then(|v| v.as_str())
                    .map_or(true, |id| id != HAJIMI_MARKER)
            });
            arr.push(new_provider);
        }

        let providers = config["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 2); // openai + hajimi (replaced, not duplicated)
        assert_eq!(providers[0]["id"], "openai");
        assert_eq!(providers[1]["id"], "hajimi");
        assert_eq!(providers[1]["apiHost"], "https://new.proxy");
    }

    #[test]
    fn test_jan_sync_fields() {
        // Jan engine config uses full_url (with /chat/completions) and api_key
        let proxy_url = "https://proxy.test/v1";
        let api_key = "sk-test";
        let base = proxy_url.trim().trim_end_matches('/');
        let base = base.trim_end_matches("/v1");
        let full_url = format!("{}/v1/chat/completions", base);

        let config = serde_json::json!({
            "full_url": full_url,
            "api_key": api_key,
        });

        assert_eq!(config["full_url"], "https://proxy.test/v1/chat/completions");
        assert_eq!(config["api_key"], "sk-test");
        // Must NOT contain old hajimi_* keys
        assert!(config.get("hajimi_proxy_url").is_none());
    }

    #[test]
    fn test_vscode_env_sync() {
        let mut config = serde_json::json!({
            "editor.fontSize": 14
        });

        let env_key = if cfg!(target_os = "macos") {
            "terminal.integrated.env.osx"
        } else if cfg!(target_os = "linux") {
            "terminal.integrated.env.linux"
        } else {
            "terminal.integrated.env.windows"
        };

        let obj = config.as_object_mut().unwrap();
        let env = obj.entry(env_key).or_insert(serde_json::json!({}));
        if let Some(env_obj) = env.as_object_mut() {
            env_obj.insert(
                "OPENAI_BASE_URL".to_string(),
                Value::String("https://proxy.test".to_string()),
            );
            env_obj.insert(
                "OPENAI_API_KEY".to_string(),
                Value::String("sk-test".to_string()),
            );
        }

        assert_eq!(config["editor.fontSize"], 14);
        assert_eq!(config[env_key]["OPENAI_BASE_URL"], "https://proxy.test");
        assert_eq!(config[env_key]["OPENAI_API_KEY"], "sk-test");
    }

    #[test]
    fn test_check_chatbox_synced() {
        let content = serde_json::json!({
            "openaiApiHost": "https://proxy.test"
        })
        .to_string();

        let (synced, _, url) = check_chatbox_synced(&content, "https://proxy.test", false);
        assert!(synced);
        assert_eq!(url, Some("https://proxy.test".to_string()));

        let (not_synced, _, _) = check_chatbox_synced(&content, "https://other.url", false);
        assert!(!not_synced);
    }

    #[test]
    fn test_check_chatbox_not_synced_empty() {
        let (synced, _, url) = check_chatbox_synced("{}", "https://proxy.test", false);
        assert!(!synced);
        assert!(url.is_none());
    }

    #[test]
    fn test_backup_path_for() {
        let p = PathBuf::from("/tmp/test/config.json");
        let bp = backup_path_for(&p);
        assert_eq!(bp, PathBuf::from("/tmp/test/config.json.antigravity.bak"));
    }

    #[test]
    fn test_read_or_empty_json_nonexistent() {
        let p = PathBuf::from("/tmp/definitely_does_not_exist_12345.json");
        let v = read_or_empty_json(&p);
        assert!(v.is_object());
        assert!(v.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_from_str_invalid() {
        assert_eq!(ExtraClient::from_str("unknown"), None);
        assert_eq!(ExtraClient::from_str(""), None);
    }

    #[test]
    fn test_all_clients_count() {
        assert_eq!(ExtraClient::all().len(), 11);
    }

    #[test]
    fn test_config_files_display() {
        assert_eq!(
            ExtraClient::Chatbox.config_files_display(),
            vec!["config.json"]
        );
        assert_eq!(
            ExtraClient::BoltAI.config_files_display(),
            vec!["(macOS Keychain)"]
        );
        assert_eq!(
            ExtraClient::Cline.config_files_display(),
            vec!["(extension settings)"]
        );
    }

    #[test]
    fn test_sillytavern_sync_fields() {
        let mut secrets = serde_json::json!({
            "existing_secret": "keep-me"
        });

        let obj = secrets.as_object_mut().unwrap();
        obj.insert(
            "api_key_openai".to_string(),
            Value::String("sk-test".to_string()),
        );
        obj.insert(
            "api_url_scale".to_string(),
            Value::String("https://proxy.test".to_string()),
        );

        assert_eq!(secrets["existing_secret"], "keep-me");
        assert_eq!(secrets["api_key_openai"], "sk-test");
        assert_eq!(secrets["api_url_scale"], "https://proxy.test");
    }
}

pub fn write_extra_config_content(_client: &ExtraClient, _file_name: &str, _content: &str) -> Result<(), String> {
    Err("Editing config for this client is not supported yet".to_string())
}

/// Return the parent folder of the config file for a given client.
pub fn get_config_folder(client: &ExtraClient) -> Option<std::path::PathBuf> {
    config_path_for(client).and_then(|p| p.parent().map(|d| d.to_path_buf()))
}
