use fs2::FileExt;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::error::{Result, SyncError};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 命令执行错误类型
#[derive(Debug)]
pub enum CommandError {
    NotFound,
    PermissionDenied,
    Timeout,
    ExecutionFailed(String),
}

/// Extract a version string from raw CLI output.
/// Handles formats like "claude/2.1.2 (Claude Code)", "codex-cli 0.86.0", "v2.0.1"
pub fn extract_version(raw: &str) -> String {
    let trimmed = raw.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    for part in &parts {
        // Format: "tool/1.2.3"
        if let Some(slash_idx) = part.find('/') {
            let after = &part[slash_idx + 1..];
            if is_version_like(after) {
                return after.to_string();
            }
        }
        // Format: "1.2.3" (standalone)
        if is_version_like(part) {
            return part.to_string();
        }
    }

    // Fallback: extract first sequence of digits and dots
    let version_chars: String = trimmed
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    if !version_chars.is_empty() && version_chars.contains('.') {
        return version_chars;
    }

    "unknown".to_string()
}

fn is_version_like(s: &str) -> bool {
    s.chars().next().map_or(false, |c| c.is_ascii_digit())
        && s.contains('.')
        && s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Search for an executable in PATH, handling platform differences.
pub fn find_in_path(executable: &str) -> Option<PathBuf> {
    let path_var = env::var("PATH").ok()?;

    #[cfg(target_os = "windows")]
    {
        let extensions = ["exe", "cmd", "bat"];
        for dir in path_var.split(';') {
            for ext in &extensions {
                let full_path = PathBuf::from(dir).join(format!("{}.{}", executable, ext));
                if full_path.exists() {
                    return Some(full_path);
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        for dir in path_var.split(':') {
            let full_path = PathBuf::from(dir).join(executable);
            if full_path.exists() {
                return Some(full_path);
            }
        }
    }

    None
}

/// Search common user and system binary locations on Unix.
#[cfg(not(target_os = "windows"))]
pub fn find_in_common_paths(executable: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    let mut candidates = vec![
        home.join(".local/bin"),
        home.join(".bun/bin"),
        home.join(".bun/install/global/node_modules/.bin"),
        home.join(".npm-global/bin"),
        home.join(".volta/bin"),
        home.join(".opencode/bin"),
        home.join("bin"),
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
    ];

    // Scan nvm directories
    let nvm_base = home.join(".nvm/versions/node");
    if nvm_base.exists() {
        if let Ok(entries) = fs::read_dir(&nvm_base) {
            for entry in entries.flatten() {
                let bin_path = entry.path().join("bin");
                if bin_path.exists() {
                    candidates.push(bin_path);
                }
            }
        }
    }

    // Scan fnm directories
    for fnm_dir in &[
        home.join(".fnm/node-versions"),
        home.join("Library/Application Support/fnm/node-versions"),
    ] {
        if fnm_dir.exists() {
            if let Ok(entries) = fs::read_dir(fnm_dir) {
                for entry in entries.flatten() {
                    let bin_path = entry.path().join("installation/bin");
                    if bin_path.exists() {
                        candidates.push(bin_path);
                    }
                }
            }
        }
    }

    for dir in &candidates {
        let full_path = dir.join(executable);
        if full_path.exists() {
            tracing::debug!("[utils] Found {} at {:?}", executable, full_path);
            return Some(full_path);
        }
    }

    None
}

/// Search common Windows binary locations.
#[cfg(target_os = "windows")]
pub fn find_in_common_paths(executable: &str) -> Option<PathBuf> {
    if let Ok(app_data) = env::var("APPDATA") {
        for ext in &["cmd", "exe"] {
            let path = PathBuf::from(&app_data)
                .join("npm")
                .join(format!("{}.{}", executable, ext));
            if path.exists() {
                return Some(path);
            }
        }
    }
    if let Ok(local) = env::var("LOCALAPPDATA") {
        for ext in &["cmd", "exe"] {
            let path = PathBuf::from(&local)
                .join("pnpm")
                .join(format!("{}.{}", executable, ext));
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// Resolve an executable path: first check PATH, then common locations.
pub fn resolve_executable(name: &str) -> Option<PathBuf> {
    if let Some(path) = find_in_path(name) {
        tracing::debug!("[utils] Found {} in PATH: {:?}", name, path);
        return Some(path);
    }
    find_in_common_paths(name)
}

/// Run `executable --version` and return the parsed version string.
/// Enhanced with detailed error reporting.
pub fn get_cli_version(executable: &PathBuf) -> Option<String> {
    let mut cmd = Command::new(executable);
    cmd.arg("--version");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    match cmd.output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let raw = if stdout.trim().is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            Some(extract_version(&raw))
        }
        Ok(output) => {
            tracing::warn!(
                "[utils] Command failed with exit code {:?}: {:?}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            );
            None
        }
        Err(e) => {
            tracing::warn!("[utils] Failed to execute {:?}: {}", executable, e);
            None
        }
    }
}

/// Create a backup of a file with the given suffix. Only creates backup once
/// (does not overwrite existing backup).
pub fn create_backup(path: &PathBuf, suffix: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| SyncError::Other("Invalid file path".to_string()))?
        .to_string_lossy();
    let backup_path = path.with_file_name(format!("{}{}", file_name, suffix));
    if backup_path.exists() {
        return Ok(());
    }
    fs::copy(path, &backup_path).map_err(|e| SyncError::FileWriteFailed {
        path: backup_path.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;
    tracing::info!("[backup] Created: {:?}", backup_path);
    Ok(())
}

/// Maximum number of timestamped backups to retain per config file.
const BACKUP_RETAIN_COUNT: usize = 5;

/// Create a timestamped backup and rotate old backups (keep latest N).
/// Returns the path to the new backup file.
pub fn create_rotated_backup(path: &PathBuf, suffix: &str) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    let file_name = path
        .file_name()
        .ok_or_else(|| SyncError::Other("Invalid file path".to_string()))?
        .to_string_lossy()
        .to_string();

    let parent = path
        .parent()
        .ok_or_else(|| SyncError::Other("Invalid file path".to_string()))?;

    // Also maintain the simple .bak for quick restore (backwards compat)
    let simple_backup = path.with_file_name(format!("{}{}", file_name, suffix));
    if !simple_backup.exists() {
        fs::copy(path, &simple_backup).map_err(|e| SyncError::FileWriteFailed {
            path: simple_backup.to_string_lossy().to_string(),
            reason: e.to_string(),
        })?;
    }

    // Create timestamped backup: filename.20260218_153045.bak
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}.{}{}", file_name, timestamp, suffix);
    let backup_path = parent.join(&backup_name);

    fs::copy(path, &backup_path).map_err(|e| SyncError::FileWriteFailed {
        path: backup_path.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;
    tracing::info!("[backup] Created rotated backup: {:?}", backup_path);

    // Cleanup: keep only the latest BACKUP_RETAIN_COUNT timestamped backups
    cleanup_old_backups(parent, &file_name, suffix)?;

    Ok(Some(backup_path))
}

/// Remove old timestamped backups, keeping the newest `BACKUP_RETAIN_COUNT`.
fn cleanup_old_backups(dir: &std::path::Path, base_name: &str, suffix: &str) -> Result<()> {
    let prefix = format!("{}.", base_name);
    let suffix_str = suffix.to_string();

    let mut backups: Vec<_> = fs::read_dir(dir)
        .map_err(|e| SyncError::Other(format!("Failed to read dir: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            // Match pattern: base_name.TIMESTAMP.suffix (e.g. settings.json.20260218_153045.antigravity.bak)
            name.starts_with(&prefix) && name.ends_with(&suffix_str) && name != format!("{}{}", base_name, suffix_str)
        })
        .collect();

    if backups.len() <= BACKUP_RETAIN_COUNT {
        return Ok(());
    }

    // Sort by modification time (oldest first)
    backups.sort_by_key(|entry| {
        entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
    });

    let remove_count = backups.len() - BACKUP_RETAIN_COUNT;
    for entry in backups.into_iter().take(remove_count) {
        if let Err(e) = fs::remove_file(entry.path()) {
            tracing::warn!("[backup] Failed to remove old backup {:?}: {}", entry.path(), e);
        } else {
            tracing::info!("[backup] Removed old backup: {:?}", entry.path());
        }
    }

    Ok(())
}

/// Atomically write content to a file using a temp file + rename pattern.
/// Enhanced with retry mechanism for Windows file locking issues.
pub fn atomic_write(target: &PathBuf, content: &str) -> Result<()> {
    atomic_write_with_retry(target, content, 5)
}

/// Atomically write with configurable retry count.
pub fn atomic_write_with_retry(target: &PathBuf, content: &str, max_retries: u32) -> Result<()> {
    #[cfg(target_os = "windows")]
    crate::system_check::check_path_length(target)?;

    let tmp_path = target.with_extension("tmp");

    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| SyncError::DirectoryCreationFailed {
            path: parent.to_string_lossy().to_string(),
            reason: e.to_string(),
        })?;
    }

    for attempt in 0..max_retries {
        match try_atomic_write(&tmp_path, target, content) {
            Ok(_) => {
                tracing::debug!("[atomic_write] Success on attempt {}", attempt + 1);
                return Ok(());
            }
            Err(e) if attempt < max_retries - 1 => {
                let wait_ms = 100 * (attempt + 1) as u64;
                tracing::warn!(
                    "[atomic_write] Attempt {} failed: {}. Retrying in {}ms...",
                    attempt + 1,
                    e,
                    wait_ms
                );
                std::thread::sleep(Duration::from_millis(wait_ms));
            }
            Err(e) => {
                // Final attempt failed
                let _ = fs::remove_file(&tmp_path); // Cleanup
                return Err(e);
            }
        }
    }

    Err(SyncError::Timeout {
        operation: format!("write file: {}", target.display()),
        seconds: ((max_retries * 100) / 1000) as u64,
    })
}

fn try_atomic_write(tmp_path: &PathBuf, target: &PathBuf, content: &str) -> Result<()> {
    // Write to temp file
    fs::write(tmp_path, content).map_err(|e| {
        let _ = fs::remove_file(tmp_path);

        // 检测具体错误类型
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            SyncError::PermissionDenied {
                path: tmp_path.to_string_lossy().to_string(),
            }
        } else {
            SyncError::FileWriteFailed {
                path: tmp_path.to_string_lossy().to_string(),
                reason: e.to_string(),
            }
        }
    })?;

    // Rename to target
    fs::rename(tmp_path, target).map_err(|e| {
        let _ = fs::remove_file(tmp_path);

        if e.kind() == std::io::ErrorKind::PermissionDenied {
            SyncError::PermissionDenied {
                path: target.to_string_lossy().to_string(),
            }
        } else {
            SyncError::FileWriteFailed {
                path: target.to_string_lossy().to_string(),
                reason: format!("Rename failed: {}", e),
            }
        }
    })?;

    Ok(())
}

/// 带文件锁的原子写入（防止并发修改）
pub fn atomic_write_with_lock(target: &PathBuf, content: &str) -> Result<()> {
    let lock_path = target.with_extension("lock");

    // 创建锁文件
    let lock_file = fs::File::create(&lock_path).map_err(|e| SyncError::FileWriteFailed {
        path: lock_path.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;

    // 尝试获取独占锁（最多等待5秒）
    for attempt in 0..50 {
        match lock_file.try_lock_exclusive() {
            Ok(_) => {
                // 获取锁成功，执行写入
                let result = atomic_write(target, content);

                // 释放锁
                let _ = fs2::FileExt::unlock(&lock_file);
                let _ = fs::remove_file(&lock_path);

                return result;
            }
            Err(_) if attempt < 49 => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => {
                return Err(SyncError::FileLocked {
                    path: target.to_string_lossy().to_string(),
                });
            }
        }
    }

    Err(SyncError::FileLocked {
        path: target.to_string_lossy().to_string(),
    })
}

/// Serialize a serde_json::Value to pretty JSON.
pub fn to_json_pretty(value: &Value) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(|e| SyncError::JsonParseFailed {
        path: "in-memory".to_string(),
        reason: e.to_string(),
    })
}

/// Validate a URL string (basic check: must start with http:// or https://)
pub fn validate_url(url: &str) -> Result<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(SyncError::InvalidUrl {
            url: "(empty)".to_string(),
        });
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(SyncError::InvalidUrl {
            url: trimmed.to_string(),
        });
    }
    Ok(())
}

/// 验证并修复损坏的JSON配置
pub fn validate_and_repair_json(path: &PathBuf, backup_suffix: &str) -> Result<Value> {
    let content = fs::read_to_string(path).map_err(|e| SyncError::FileReadFailed {
        path: path.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;

    match serde_json::from_str::<Value>(&content) {
        Ok(json) => Ok(json),
        Err(e) => {
            tracing::error!("[validate_json] Config corrupted: {}", e);

            // 尝试从备份恢复
            let backup_path = path.with_file_name(format!(
                "{}{}",
                path.file_name().unwrap().to_string_lossy(),
                backup_suffix
            ));

            if backup_path.exists() {
                tracing::info!("[validate_json] Attempting to restore from backup...");
                let backup_content =
                    fs::read_to_string(&backup_path).map_err(|e| SyncError::ConfigCorrupted {
                        path: path.to_string_lossy().to_string(),
                        reason: format!("Backup also unreadable: {}", e),
                    })?;

                match serde_json::from_str::<Value>(&backup_content) {
                    Ok(json) => {
                        // 备份有效，恢复它
                        fs::copy(&backup_path, path).map_err(|e| SyncError::FileWriteFailed {
                            path: path.to_string_lossy().to_string(),
                            reason: e.to_string(),
                        })?;
                        tracing::info!("[validate_json] Restored from backup successfully");
                        Ok(json)
                    }
                    Err(backup_err) => Err(SyncError::ConfigCorrupted {
                        path: path.to_string_lossy().to_string(),
                        reason: format!("Original: {}. Backup: {}", e, backup_err),
                    }),
                }
            } else {
                Err(SyncError::ConfigCorrupted {
                    path: path.to_string_lossy().to_string(),
                    reason: format!("JSON parse error: {}. No backup found.", e),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_slash_format() {
        assert_eq!(extract_version("claude/2.1.2 (Claude Code)"), "2.1.2");
        assert_eq!(extract_version("opencode/1.2.3"), "1.2.3");
    }

    #[test]
    fn test_extract_version_space_format() {
        assert_eq!(extract_version("codex-cli 0.86.0\n"), "0.86.0");
    }

    #[test]
    fn test_extract_version_v_prefix() {
        assert_eq!(extract_version("v2.0.1"), "2.0.1");
    }

    #[test]
    fn test_extract_version_unknown() {
        assert_eq!(extract_version("some random text"), "unknown");
        assert_eq!(extract_version(""), "unknown");
    }

    #[test]
    fn test_is_version_like() {
        assert!(is_version_like("1.2.3"));
        assert!(is_version_like("0.86.0"));
        assert!(!is_version_like("abc"));
        assert!(!is_version_like("v1.2.3")); // starts with 'v'
        assert!(!is_version_like("123")); // no dot
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://localhost:3000").is_ok());
        assert!(validate_url("https://free.aipro.love/v1").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("not-a-url").is_err());
    }
}
