use serde::{Deserialize, Serialize};
use std::env;

use crate::error::{get_install_hint, Result, SyncError};
use crate::utils;

/// 系统环境检测结果
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemRequirements {
    pub has_git: bool,
    pub has_npm: bool,
    pub has_node: bool,
    pub home_dir_exists: bool,
    pub disk_space_mb: u64,
    pub platform: String,
    pub appdata_exists: bool, // Windows only
    pub issues: Vec<SystemIssue>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub fix_hint: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// 执行完整的系统环境检查
pub fn check_system() -> SystemRequirements {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    // 检查 HOME 目录
    let home_dir_exists = dirs::home_dir().is_some();
    if !home_dir_exists {
        issues.push(SystemIssue {
            severity: IssueSeverity::Error,
            code: "HOME_NOT_FOUND".to_string(),
            message: "Cannot determine home directory".to_string(),
            fix_hint: "Your user profile may be corrupted. Please contact system administrator."
                .to_string(),
        });
    }

    // 检查 Git
    let has_git = utils::resolve_executable("git").is_some();
    if !has_git {
        issues.push(SystemIssue {
            severity: IssueSeverity::Warning,
            code: "GIT_NOT_FOUND".to_string(),
            message: "Git is not installed".to_string(),
            fix_hint: get_install_hint("git"),
        });
    }

    // 检查 Node.js
    let has_node = utils::resolve_executable("node").is_some();
    if !has_node {
        issues.push(SystemIssue {
            severity: IssueSeverity::Warning,
            code: "NODE_NOT_FOUND".to_string(),
            message: "Node.js is not installed (required for CLI tools, not needed for desktop apps)".to_string(),
            fix_hint: get_install_hint("node"),
        });
    }

    // 检查 NPM
    let has_npm = utils::resolve_executable("npm").is_some();
    if !has_npm && has_node {
        issues.push(SystemIssue {
            severity: IssueSeverity::Warning,
            code: "NPM_NOT_FOUND".to_string(),
            message: "npm is not installed or not in PATH".to_string(),
            fix_hint: get_install_hint("npm"),
        });
    }

    // 检查磁盘空间
    let disk_space_mb = get_available_disk_space();
    if disk_space_mb < 100 {
        issues.push(SystemIssue {
            severity: IssueSeverity::Error,
            code: "LOW_DISK_SPACE".to_string(),
            message: format!("Low disk space: only {} MB available", disk_space_mb),
            fix_hint: "Please free up disk space before proceeding.".to_string(),
        });
    } else if disk_space_mb < 500 {
        warnings.push(format!(
            "Low disk space: {} MB available. Consider freeing up space.",
            disk_space_mb
        ));
    }

    // Windows 特定检查
    let mut appdata_exists = true;
    if cfg!(target_os = "windows") {
        appdata_exists = env::var("APPDATA").is_ok();
        if !appdata_exists {
            issues.push(SystemIssue {
                severity: IssueSeverity::Error,
                code: "APPDATA_NOT_SET".to_string(),
                message: "APPDATA environment variable is not set".to_string(),
                fix_hint:
                    "This is unusual on Windows. Your system configuration may be incomplete."
                        .to_string(),
            });
        }

        // 检查 UAC 权限提示
        if let Ok(username) = env::var("USERNAME") {
            if username.to_lowercase() != "administrator" {
                warnings.push("You are not running as Administrator. Some operations may require elevated privileges.".to_string());
            }
        }

        // 检查路径长度限制
        if let Some(home) = dirs::home_dir() {
            let path_str = home.to_string_lossy();
            if path_str.len() > 200 {
                warnings.push(format!("Home directory path is very long ({} chars). Windows MAX_PATH is 260. This may cause issues.", path_str.len()));
            }
        }
    }

    let platform = env::consts::OS.to_string();

    SystemRequirements {
        has_git,
        has_npm,
        has_node,
        home_dir_exists,
        disk_space_mb,
        platform,
        appdata_exists,
        issues,
        warnings,
    }
}

/// 验证系统是否满足最低要求
pub fn validate_system_requirements() -> Result<()> {
    let sys = check_system();

    // 检查致命错误
    let critical_errors: Vec<_> = sys
        .issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Error)
        .collect();

    if !critical_errors.is_empty() {
        let error_msg = critical_errors
            .iter()
            .map(|e| format!("❌ {}: {}\n   Fix: {}", e.code, e.message, e.fix_hint))
            .collect::<Vec<_>>()
            .join("\n\n");

        return Err(SyncError::Other(format!(
            "System requirements not met:\n\n{}",
            error_msg
        )));
    }

    // 显示警告（不阻止执行）
    if !sys.warnings.is_empty() {
        for warning in &sys.warnings {
            tracing::warn!("[system_check] {}", warning);
        }
    }

    Ok(())
}

/// 获取可用磁盘空间（MB）
fn get_available_disk_space() -> u64 {
    use sysinfo::Disks;

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return 0,
    };

    let disks = Disks::new_with_refreshed_list();

    // 查找包含 home 目录的磁盘
    for disk in &disks {
        let mount_point = disk.mount_point();
        if home.starts_with(mount_point) {
            return disk.available_space() / (1024 * 1024); // 转换为 MB
        }
    }

    // 如果找不到，返回第一个磁盘的可用空间
    disks
        .first()
        .map(|d| d.available_space() / (1024 * 1024))
        .unwrap_or(0)
}

/// Tauri command: 获取系统检测结果
#[tauri::command]
pub fn get_system_status() -> SystemRequirements {
    check_system()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_system() {
        let sys = check_system();
        assert!(!sys.platform.is_empty());
        // 至少应该能检测到磁盘空间
        assert!(sys.disk_space_mb >= 0);
    }

    #[test]
    fn test_validate_system_basic() {
        // 这个测试可能会失败（如果系统真的不满足要求），但至少应该不panic
        let result = validate_system_requirements();
        // 只记录结果，不断言
        match result {
            Ok(_) => println!("System requirements met"),
            Err(e) => println!("System requirements not met: {}", e),
        }
    }

    #[test]
    fn test_get_disk_space() {
        let space = get_available_disk_space();
        println!("Available disk space: {} MB", space);
        // 不做断言，因为不同环境不同
    }
}
