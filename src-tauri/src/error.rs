use thiserror::Error;

/// 主错误类型，提供详细的错误信息和用户友好的修复建议
#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Home directory not found. Please ensure your user profile is set up correctly.")]
    HomeDirectoryNotFound,

    #[error("Insufficient disk space: need {required} MB, but only {available} MB available")]
    InsufficientDiskSpace { required: u64, available: u64 },

    #[error("Permission denied when accessing: {path}\n\nOn Windows: Try running as Administrator\nOn macOS/Linux: Check file permissions with 'ls -la {path}'")]
    PermissionDenied { path: String },

    #[error("CLI tool '{name}' is not installed.\n\nInstall instructions:\n{install_hint}")]
    CliNotInstalled { name: String, install_hint: String },

    #[error("Config file corrupted: {path}\nReason: {reason}\n\nThe backup file will be used for recovery.")]
    ConfigCorrupted { path: String, reason: String },

    #[error("Required dependency '{tool}' is missing.\n\nInstall instructions:\n{install_hint}")]
    DependencyMissing { tool: String, install_hint: String },

    #[error("Failed to create directory: {path}\nReason: {reason}")]
    DirectoryCreationFailed { path: String, reason: String },

    #[error("Failed to read file: {path}\nReason: {reason}")]
    FileReadFailed { path: String, reason: String },

    #[error("Failed to write file: {path}\nReason: {reason}\n\nPossible causes:\n- File is locked by another process\n- Antivirus software blocking the operation\n- Insufficient permissions")]
    FileWriteFailed { path: String, reason: String },

    #[error("Failed to parse JSON in {path}: {reason}")]
    JsonParseFailed { path: String, reason: String },

    #[error("Failed to execute command: {command}\nReason: {reason}")]
    CommandExecutionFailed { command: String, reason: String },

    #[error("Backup file not found for {path}")]
    BackupNotFound { path: String },

    #[error("File is locked by another process: {path}\n\nPlease close any applications using this file and try again.")]
    FileLocked { path: String },

    #[error("Operation timed out after {seconds} seconds: {operation}")]
    Timeout { operation: String, seconds: u64 },

    #[error("Invalid URL: {url}\n\nURL must start with http:// or https://")]
    InvalidUrl { url: String },

    #[error("Environment variable '{var}' is not set.\n\nThis is required on Windows. Please check your system settings.")]
    EnvVarNotSet { var: String },

    #[error("Path too long (Windows MAX_PATH limit): {path}\n\nPath length: {length}, Maximum: 260\n\nConsider moving the project to a shorter path.")]
    PathTooLong { path: String, length: usize },

    #[error("{0}")]
    Other(String),
}

impl SyncError {
    /// 获取用户友好的错误码
    pub fn code(&self) -> &'static str {
        match self {
            Self::HomeDirectoryNotFound => "HOME_NOT_FOUND",
            Self::InsufficientDiskSpace { .. } => "DISK_FULL",
            Self::PermissionDenied { .. } => "PERMISSION_DENIED",
            Self::CliNotInstalled { .. } => "CLI_NOT_INSTALLED",
            Self::ConfigCorrupted { .. } => "CONFIG_CORRUPTED",
            Self::DependencyMissing { .. } => "DEPENDENCY_MISSING",
            Self::DirectoryCreationFailed { .. } => "DIR_CREATE_FAILED",
            Self::FileReadFailed { .. } => "FILE_READ_FAILED",
            Self::FileWriteFailed { .. } => "FILE_WRITE_FAILED",
            Self::JsonParseFailed { .. } => "JSON_PARSE_FAILED",
            Self::CommandExecutionFailed { .. } => "COMMAND_FAILED",
            Self::BackupNotFound { .. } => "BACKUP_NOT_FOUND",
            Self::FileLocked { .. } => "FILE_LOCKED",
            Self::Timeout { .. } => "TIMEOUT",
            Self::InvalidUrl { .. } => "INVALID_URL",
            Self::EnvVarNotSet { .. } => "ENV_VAR_NOT_SET",
            Self::PathTooLong { .. } => "PATH_TOO_LONG",
            Self::Other(_) => "UNKNOWN",
        }
    }

    /// 判断是否可以自动恢复
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::FileLocked { .. }
                | Self::FileWriteFailed { .. }
                | Self::Timeout { .. }
                | Self::ConfigCorrupted { .. }
        )
    }
}

/// 便捷的Result类型别名
pub type Result<T> = std::result::Result<T, SyncError>;

/// 实现From转换，让SyncError可以自动转为String
impl From<SyncError> for String {
    fn from(err: SyncError) -> String {
        err.to_string()
    }
}

/// 辅助函数：获取CLI安装提示
pub fn get_install_hint(tool: &str) -> String {
    match tool {
        "git" => {
            if cfg!(target_os = "windows") {
                "Download from: https://git-scm.com/download/win\nOr use: winget install Git.Git".to_string()
            } else if cfg!(target_os = "macos") {
                "Run: brew install git\nOr download from: https://git-scm.com/download/mac".to_string()
            } else {
                "Run: sudo apt-get install git (Ubuntu/Debian)\nOr: sudo yum install git (CentOS/RHEL)".to_string()
            }
        }
        "npm" | "node" => {
            if cfg!(target_os = "windows") {
                "Download Node.js from: https://nodejs.org/\nOr use: winget install OpenJS.NodeJS".to_string()
            } else if cfg!(target_os = "macos") {
                "Run: brew install node\nOr download from: https://nodejs.org/".to_string()
            } else {
                "Run: sudo apt-get install nodejs npm (Ubuntu/Debian)\nOr use nvm: curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash".to_string()
            }
        }
        "claude" => {
            "Install Claude Code:\nnpm install -g @anthropic-ai/claude-code\n\nOr follow: https://docs.anthropic.com/en/docs/claude-code".to_string()
        }
        "codex" => {
            "Install Codex CLI:\nnpm install -g @openai/codex\n\nOr follow: https://github.com/openai/codex".to_string()
        }
        "gemini" => {
            "Install Gemini CLI:\nnpm install -g @google/gemini-cli\n\nOr follow: https://github.com/google-gemini/gemini-cli".to_string()
        }
        "opencode" => {
            "Install OpenCode from GitHub:\nhttps://github.com/anomalyco/opencode\n\nSee the README for installation instructions.".to_string()
        }
        "droid" => {
            "Download Droid from: https://factory.ai".to_string()
        }
        "cursor" => {
            "Download Cursor from: https://cursor.com/downloads".to_string()
        }
        "chatbox" => {
            "Download Chatbox from: https://chatboxai.app".to_string()
        }
        "cherry-studio" => {
            "Download Cherry Studio from: https://cherry-ai.com".to_string()
        }
        "jan" => {
            "Download Jan from: https://jan.ai/download".to_string()
        }
        "cline" => {
            "Install Cline extension in VS Code:\ncode --install-extension saoudrizwan.claude-dev".to_string()
        }
        "roo-code" => {
            "Install Roo Code extension in VS Code:\ncode --install-extension rooveterinaryinc.roo-cline".to_string()
        }
        "kilo-code" => {
            "Install Kilo Code extension in VS Code:\ncode --install-extension kilocode.kilo-code".to_string()
        }
        "sillytavern" => {
            "Install SillyTavern:\ngit clone https://github.com/SillyTavern/SillyTavern\n\nSee: https://docs.sillytavern.app/installation/".to_string()
        }
        "lobechat" => {
            "Download LobeChat from: https://lobehub.com/download".to_string()
        }
        "boltai" => {
            "Download BoltAI from: https://boltai.com (macOS only)".to_string()
        }
        _ => format!("Search for '{tool} installation guide' for your platform"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = SyncError::HomeDirectoryNotFound;
        assert_eq!(err.code(), "HOME_NOT_FOUND");

        let err = SyncError::CliNotInstalled {
            name: "test".to_string(),
            install_hint: "hint".to_string(),
        };
        assert_eq!(err.code(), "CLI_NOT_INSTALLED");
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(SyncError::FileLocked {
            path: "test".to_string()
        }
        .is_recoverable());
        assert!(!SyncError::HomeDirectoryNotFound.is_recoverable());
    }

    #[test]
    fn test_install_hints() {
        let hint = get_install_hint("git");
        assert!(!hint.is_empty());
        assert!(hint.contains("git") || hint.contains("Git"));
    }
}
