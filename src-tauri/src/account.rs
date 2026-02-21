use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Persistent account state managed by Tauri
pub struct AccountState {
    pub inner: Mutex<AccountStateInner>,
}

pub struct AccountStateInner {
    pub session_cookie: Option<String>,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub base_url: Option<String>,
}

impl AccountState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(AccountStateInner {
                session_cookie: None,
                user_id: None,
                username: None,
                base_url: None,
            }),
        }
    }
}

// ── Response types from new-api ──

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    message: Option<String>,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct LoginData {
    id: i64,
    username: String,
    display_name: Option<String>,
    #[allow(dead_code)]
    role: Option<i64>,
    #[allow(dead_code)]
    status: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PageData<T> {
    #[allow(dead_code)]
    page: Option<i64>,
    #[allow(dead_code)]
    page_size: Option<i64>,
    #[allow(dead_code)]
    total: Option<i64>,
    items: Option<Vec<T>>,
}

#[derive(Debug, Deserialize)]
struct RawToken {
    id: Option<i64>,
    name: Option<String>,
    key: Option<String>,
    status: Option<i64>,
    used_quota: Option<i64>,
    remain_quota: Option<i64>,
    unlimited_quota: Option<bool>,
    expired_time: Option<i64>,
    model_limits_enabled: Option<bool>,
    model_limits: Option<String>,
}

// ── Types returned to frontend ──

#[derive(Debug, Serialize, Clone)]
pub struct PlatformInfo {
    pub system_name: String,
    pub version: String,
    pub register_enabled: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct AccountInfo {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    /// Session cookie returned on login, None on session check
    pub session_cookie: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ApiTokenInfo {
    pub id: i64,
    pub name: String,
    pub key: String,
    pub status: i64,
    pub used_quota: i64,
    pub remain_quota: i64,
    pub unlimited_quota: bool,
    pub expired_time: i64,
    pub model_limits_enabled: bool,
    pub model_limits: Vec<String>,
}

// ── Helper ──

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))
}

fn normalize_base(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_string()
}

/// Extract `session=xxx` from Set-Cookie headers
fn extract_session_cookie(response: &reqwest::Response) -> Option<String> {
    for val in response.headers().get_all("set-cookie") {
        if let Ok(s) = val.to_str() {
            // Cookie value looks like: session=xxxx; Path=/; ...
            if s.starts_with("session=") {
                // Take only the key=value part before the first ';'
                let cookie_val = s.split(';').next().unwrap_or(s);
                return Some(cookie_val.to_string());
            }
        }
    }
    None
}

fn auth_headers(session_cookie: &str, user_id: i64) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(session_cookie)
            .map_err(|e| format!("Invalid cookie value: {e}"))?,
    );
    headers.insert(
        "New-Api-User",
        HeaderValue::from_str(&user_id.to_string())
            .map_err(|e| format!("Invalid user id header: {e}"))?,
    );
    Ok(headers)
}

/// Helper: lock the AccountState mutex with poison recovery.
/// If a thread panicked while holding the lock, we recover the inner data
/// instead of permanently locking out all subsequent callers.
fn lock_account(state: &AccountState) -> Result<std::sync::MutexGuard<'_, AccountStateInner>, String> {
    state.inner.lock().or_else(|poisoned| {
        tracing::warn!("AccountState mutex was poisoned, recovering");
        Ok(poisoned.into_inner())
    })
}

// ── Tauri commands ──

/// Check platform info (public, no auth needed)
#[tauri::command]
pub async fn check_platform(base_url: String) -> Result<PlatformInfo, String> {
    let base = normalize_base(&base_url);
    let client = build_client()?;

    let response = client
        .get(format!("{base}/api/status"))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "CONNECT_TIMEOUT".to_string()
            } else {
                "CONNECT_FAILED".to_string()
            }
        })?;

    if !response.status().is_success() {
        return Err(format!("Server returned {}", response.status()));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Invalid response: {e}"))?;

    let data = body.get("data").ok_or("Invalid response format")?;

    Ok(PlatformInfo {
        system_name: data
            .get("system_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        version: data
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        register_enabled: data
            .get("password_register_enabled")
            .and_then(|v| v.as_bool())
            // fallback: check old field name
            .or_else(|| data.get("register_enabled").and_then(|v| v.as_bool()))
            .unwrap_or(false),
    })
}

/// Login with username/password, store session in state
#[tauri::command]
pub async fn account_login(
    base_url: String,
    username: String,
    password: String,
    state: tauri::State<'_, AccountState>,
) -> Result<AccountInfo, String> {
    let base = normalize_base(&base_url);
    let client = build_client()?;

    let response = client
        .post(format!("{base}/api/user/login"))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "CONNECT_TIMEOUT".to_string()
            } else {
                "CONNECT_FAILED".to_string()
            }
        })?;

    // Extract session cookie before consuming response body
    let session_cookie = extract_session_cookie(&response);

    let status_code = response.status();
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|_| "INVALID_RESPONSE".to_string())?;

    // Check for 2FA requirement
    if let Some(true) = body.get("data").and_then(|d| d.get("require_2fa")).and_then(|v| v.as_bool()) {
        return Err("REQUIRE_2FA".to_string());
    }

    let success = body.get("success").and_then(|v| v.as_bool()).unwrap_or(false);

    if !success {
        let server_msg = body.get("message").and_then(|v| v.as_str()).unwrap_or("");
        // Map common new-api error messages to error codes
        if server_msg.contains("密码") || server_msg.contains("password") || server_msg.contains("用户名") || server_msg.contains("username") {
            return Err("WRONG_CREDENTIALS".to_string());
        }
        return Err("LOGIN_FAILED".to_string());
    }

    if !status_code.is_success() {
        return Err("LOGIN_FAILED".to_string());
    }

    let data = body.get("data").ok_or("INVALID_RESPONSE")?;
    let id = data.get("id").and_then(|v| v.as_i64()).ok_or("INVALID_RESPONSE")?;
    let uname = data.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let display = data.get("display_name").and_then(|v| v.as_str()).unwrap_or(&uname).to_string();
    let user_status = data.get("status").and_then(|v| v.as_i64());

    // Check if user is disabled
    if user_status == Some(2) {
        return Err("ACCOUNT_DISABLED".to_string());
    }

    let session = session_cookie.ok_or("NO_SESSION_COOKIE")?;

    // Store in state
    {
        let mut inner = lock_account(&state)?;
        inner.session_cookie = Some(session.clone());
        inner.user_id = Some(id);
        inner.username = Some(uname.clone());
        inner.base_url = Some(base);
    }

    Ok(AccountInfo {
        user_id: id,
        username: uname,
        display_name: display,
        session_cookie: Some(session),
    })
}

/// Get all API tokens for the logged-in user
#[tauri::command]
pub async fn account_get_tokens(
    state: tauri::State<'_, AccountState>,
) -> Result<Vec<ApiTokenInfo>, String> {
    let (base, session, user_id) = {
        let inner = lock_account(&state)?;
        let base = inner.base_url.clone().ok_or("Not logged in")?;
        let session = inner.session_cookie.clone().ok_or("Not logged in")?;
        let user_id = inner.user_id.ok_or("Not logged in")?;
        (base, session, user_id)
    };

    let client = build_client()?;
    let headers = auth_headers(&session, user_id)?;

    let response = client
        .get(format!("{base}/api/token/?p=1&page_size=100"))
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch tokens: {e}"))?;

    let status_code = response.status();

    if status_code.as_u16() == 401 || status_code.as_u16() == 403 {
        // Session expired
        return Err("SESSION_EXPIRED".to_string());
    }

    if !status_code.is_success() {
        return Err(format!("Server returned {status_code}"));
    }

    let body: ApiResponse<PageData<RawToken>> = response
        .json()
        .await
        .map_err(|e| format!("Invalid response: {e}"))?;

    if !body.success {
        let msg = body.message.unwrap_or_else(|| "Failed to fetch tokens".to_string());
        return Err(msg);
    }

    let page_data = body.data.ok_or("No data in response")?;
    let items = page_data.items.unwrap_or_default();

    let tokens: Vec<ApiTokenInfo> = items
        .into_iter()
        .map(|t| {
            let model_limits: Vec<String> = t
                .model_limits
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();

            let key_raw = t.key.unwrap_or_default();
            // new-api stores key without "sk-" prefix, but returns it; ensure consistency
            let key = if key_raw.starts_with("sk-") {
                key_raw
            } else {
                format!("sk-{key_raw}")
            };

            ApiTokenInfo {
                id: t.id.unwrap_or(0),
                name: t.name.unwrap_or_default(),
                key,
                status: t.status.unwrap_or(0),
                used_quota: t.used_quota.unwrap_or(0),
                remain_quota: t.remain_quota.unwrap_or(0),
                unlimited_quota: t.unlimited_quota.unwrap_or(false),
                // new-api uses -1 for "never expires", but some versions return 0.
                // Treat both as never-expires to avoid false "expired" display.
                expired_time: match t.expired_time.unwrap_or(-1) {
                    0 | -1 => -1,
                    ts => ts,
                },
                model_limits_enabled: t.model_limits_enabled.unwrap_or(false),
                model_limits,
            }
        })
        .collect();

    Ok(tokens)
}

/// Check if session is still valid by calling GET /api/user/self
#[tauri::command]
pub async fn account_check_session(
    state: tauri::State<'_, AccountState>,
) -> Result<AccountInfo, String> {
    let (base, session, user_id) = {
        let inner = lock_account(&state)?;
        let base = inner.base_url.clone().ok_or("NOT_LOGGED_IN")?;
        let session = inner.session_cookie.clone().ok_or("NOT_LOGGED_IN")?;
        let user_id = inner.user_id.ok_or("NOT_LOGGED_IN")?;
        (base, session, user_id)
    };

    let client = build_client()?;
    let headers = auth_headers(&session, user_id)?;

    let response = client
        .get(format!("{base}/api/user/self"))
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Session check failed: {e}"))?;

    let status_code = response.status();

    if status_code.as_u16() == 401 || status_code.as_u16() == 403 {
        // Clear entire expired session
        if let Ok(mut inner) = lock_account(&state) {
            inner.session_cookie = None;
            inner.user_id = None;
            inner.username = None;
            inner.base_url = None;
        }
        return Err("SESSION_EXPIRED".to_string());
    }

    if !status_code.is_success() {
        return Err(format!("Server returned {status_code}"));
    }

    let body: ApiResponse<LoginData> = response
        .json()
        .await
        .map_err(|e| format!("Invalid response: {e}"))?;

    if !body.success {
        return Err("SESSION_EXPIRED".to_string());
    }

    let data = body.data.ok_or("SESSION_EXPIRED")?;

    Ok(AccountInfo {
        user_id: data.id,
        username: data.username.clone(),
        display_name: data.display_name.unwrap_or(data.username),
        session_cookie: None,
    })
}

/// Restore session from frontend-persisted data (called on app startup)
#[tauri::command]
pub async fn account_restore_session(
    base_url: String,
    session_cookie: String,
    user_id: i64,
    username: String,
    state: tauri::State<'_, AccountState>,
) -> Result<(), String> {
    {
        let mut inner = lock_account(&state)?;
        inner.base_url = Some(normalize_base(&base_url));
        inner.session_cookie = Some(session_cookie);
        inner.user_id = Some(user_id);
        inner.username = Some(username);
    }
    Ok(())
}

/// Logout — clear state
#[tauri::command]
pub async fn account_logout(
    state: tauri::State<'_, AccountState>,
) -> Result<(), String> {
    // Optionally call server logout
    let (base, session, user_id) = {
        let inner = lock_account(&state)?;
        (
            inner.base_url.clone(),
            inner.session_cookie.clone(),
            inner.user_id,
        )
    };

    if let (Some(base), Some(session), Some(uid)) = (base, session, user_id) {
        let client = build_client()?;
        if let Ok(headers) = auth_headers(&session, uid) {
            // Fire and forget — don't fail if server logout fails
            let _ = client
                .get(format!("{base}/api/user/logout"))
                .headers(headers)
                .send()
                .await;
        }
    }

    // Clear local state
    let mut inner = lock_account(&state)?;
    inner.session_cookie = None;
    inner.user_id = None;
    inner.username = None;
    inner.base_url = None;

    Ok(())
}
