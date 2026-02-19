use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

use crate::error::{Result, SyncError};
use crate::utils;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 安装进度状态
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub tool: String,
    pub status: InstallStatus,
    pub progress: u8, // 0-100
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstallStatus {
    Checking,
    Downloading,
    Installing,
    Completed,
    Failed,
    Skipped,
}

/// 自动安装Git（静默）
pub async fn auto_install_git() -> Result<()> {
    tracing::info!("[auto_installer] Starting automatic Git installation...");

    #[cfg(target_os = "windows")]
    {
        // Windows: 优先使用winget，fallback到chocolatey
        if check_command_exists("winget") {
            tracing::info!("[auto_installer] Using winget to install Git");
            run_silent_command(
                "winget",
                &[
                    "install",
                    "Git.Git",
                    "-e",
                    "--silent",
                    "--accept-package-agreements",
                    "--accept-source-agreements",
                ],
            )
            .await?;
        } else if check_command_exists("choco") {
            tracing::info!("[auto_installer] Using chocolatey to install Git");
            run_silent_command("choco", &["install", "git", "-y"]).await?;
        } else {
            // 下载便携版Git（无需安装）
            return download_portable_git().await;
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用Homebrew，如果没有则使用Xcode Command Line Tools
        if check_command_exists("brew") {
            tracing::info!("[auto_installer] Using Homebrew to install Git");
            run_silent_command("brew", &["install", "git"]).await?;
        } else {
            tracing::info!("[auto_installer] Installing Xcode Command Line Tools");
            run_silent_command("xcode-select", &["--install"]).await?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 自动检测包管理器
        if check_command_exists("apt-get") {
            tracing::info!("[auto_installer] Using apt-get to install Git");
            run_silent_command("sudo", &["apt-get", "update", "-qq"]).await?;
            run_silent_command("sudo", &["apt-get", "install", "-y", "git"]).await?;
        } else if check_command_exists("yum") {
            tracing::info!("[auto_installer] Using yum to install Git");
            run_silent_command("sudo", &["yum", "install", "-y", "git"]).await?;
        } else if check_command_exists("dnf") {
            tracing::info!("[auto_installer] Using dnf to install Git");
            run_silent_command("sudo", &["dnf", "install", "-y", "git"]).await?;
        } else if check_command_exists("pacman") {
            tracing::info!("[auto_installer] Using pacman to install Git");
            run_silent_command("sudo", &["pacman", "-S", "--noconfirm", "git"]).await?;
        } else {
            return Err(SyncError::Other("No package manager found".to_string()));
        }
    }

    Ok(())
}

/// 自动安装Node.js（静默）
pub async fn auto_install_nodejs() -> Result<()> {
    auto_install_nodejs_version("22").await
}

/// Ensure Node.js 22+ is available (required by OpenClaw)
async fn ensure_node22() -> Result<()> {
    if let Some(version) = get_node_major_version() {
        if version >= 22 {
            tracing::info!("[auto_installer] Node.js v{} detected, meets 22+ requirement", version);
            return Ok(());
        }
        tracing::warn!("[auto_installer] Node.js v{} detected but OpenClaw requires 22+, upgrading...", version);
    } else {
        tracing::info!("[auto_installer] Node.js not found, installing v22...");
    }
    auto_install_nodejs_version("22").await
}

/// Get the major version of installed Node.js, if any
fn get_node_major_version() -> Option<u32> {
    let output = Command::new("node")
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version_str = String::from_utf8_lossy(&output.stdout);
    // Parse "v22.12.0" -> 22
    version_str.trim().trim_start_matches('v')
        .split('.')
        .next()?
        .parse::<u32>()
        .ok()
}

/// 安装指定大版本的Node.js（静默）
async fn auto_install_nodejs_version(major: &str) -> Result<()> {
    tracing::info!("[auto_installer] Starting automatic Node.js {} installation...", major);

    #[cfg(target_os = "windows")]
    {
        if check_command_exists("winget") {
            tracing::info!("[auto_installer] Using winget to install Node.js");
            run_silent_command(
                "winget",
                &[
                    "install",
                    "OpenJS.NodeJS",
                    "-e",
                    "--silent",
                    "--accept-package-agreements",
                    "--accept-source-agreements",
                ],
            )
            .await?;
        } else if check_command_exists("choco") {
            tracing::info!("[auto_installer] Using chocolatey to install Node.js");
            run_silent_command("choco", &["install", "nodejs", "-y"]).await?;
        } else {
            return install_nodejs_standalone().await;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if check_command_exists("brew") {
            tracing::info!("[auto_installer] Using Homebrew to install Node.js");
            run_silent_command("brew", &["install", "node"]).await?;
        } else {
            return install_nodejs_standalone().await;
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 使用NodeSource官方脚本
        tracing::info!("[auto_installer] Using NodeSource to install Node.js");
        install_nodejs_nodesource().await?;
    }

    Ok(())
}

/// 自动安装CLI工具（通过npm）
pub async fn auto_install_cli_tool(tool: &str) -> Result<()> {
    tracing::info!("[auto_installer] Installing CLI tool: {}", tool);

    // 确保npm可用
    if !check_command_exists("npm") {
        tracing::warn!("[auto_installer] npm not found, installing Node.js first");
        auto_install_nodejs().await?;

        // 等待npm安装完成
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if check_command_exists("npm") {
                break;
            }
        }

        if !check_command_exists("npm") {
            return Err(SyncError::Other("Failed to install npm".to_string()));
        }
    }

    let package_name = match tool {
        "claude" => "@anthropic-ai/claude-code",
        "codex" => "@openai/codex",
        "gemini" => "@google/gemini-cli",
        // OpenClaw requires Node.js 22.12.0+, official npm package is "openclaw"
        "openclaw" => {
            ensure_node22().await?;
            "openclaw"
        }
        // OpenCode is installed from GitHub, not npm
        "opencode" => {
            return Err(SyncError::Other(
                "OpenCode must be installed from GitHub. See: https://github.com/anomalyco/opencode".to_string()
            ));
        }
        // Desktop apps — cannot be installed via npm
        "chatbox" | "cherry-studio" | "jan" | "cursor" | "lobechat" | "boltai" => {
            return Err(SyncError::Other(format!(
                "{} is a desktop application. Please download it from its official website.",
                tool
            )));
        }
        // VS Code extensions — install via `code --install-extension`
        "claude-vscode" => {
            return install_vscode_extension("anthropic.claude-code").await;
        }
        "cline" => {
            return install_vscode_extension("saoudrizwan.claude-dev").await;
        }
        "roo-code" => {
            return install_vscode_extension("rooveterinaryinc.roo-cline").await;
        }
        "kilo-code" => {
            return install_vscode_extension("kilocode.kilo-code").await;
        }
        // SillyTavern is a Node.js app, not an npm global package
        "sillytavern" => {
            return Err(SyncError::Other(
                "SillyTavern must be installed via git clone. See: https://docs.sillytavern.app/installation/".to_string()
            ));
        }
        // Droid has no public npm package
        "droid" => {
            return Err(SyncError::Other(
                "Droid must be installed from https://factory.ai".to_string(),
            ));
        }
        _ => tool,
    };

    tracing::info!("[auto_installer] Installing npm package: {}", package_name);

    let args = vec!["install", "-g", package_name, "--silent", "--no-progress"];
    run_silent_command("npm", &args).await?;

    Ok(())
}

/// Install a VS Code extension via the `code` CLI
async fn install_vscode_extension(extension_id: &str) -> Result<()> {
    if !check_command_exists("code") {
        return Err(SyncError::Other(
            "VS Code CLI ('code') not found. Please install VS Code first.".to_string(),
        ));
    }
    tracing::info!(
        "[auto_installer] Installing VS Code extension: {}",
        extension_id
    );
    run_silent_command("code", &["--install-extension", extension_id, "--force"]).await
}

/// 检查命令是否存在
fn check_command_exists(cmd: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let extensions = ["exe", "cmd", "bat"];
        for ext in &extensions {
            let result = Command::new("where")
                .arg(format!("{}.{}", cmd, ext))
                .creation_flags(CREATE_NO_WINDOW)
                .output();

            if let Ok(output) = result {
                if output.status.success() {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// 静默执行命令（带超时）
async fn run_silent_command(cmd: &str, args: &[&str]) -> Result<()> {
    run_silent_command_with_timeout(cmd, args, Duration::from_secs(120)).await
}

/// 静默执行命令（可配置超时时间）
async fn run_silent_command_with_timeout(
    cmd: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<()> {
    tracing::debug!(
        "[auto_installer] Running: {} {:?} (timeout: {:?})",
        cmd,
        args,
        timeout
    );

    let cmd_str = cmd.to_string();
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let task = tokio::task::spawn_blocking(move || {
        let mut command = Command::new(&cmd_str);
        command.args(&args_vec);

        #[cfg(target_os = "windows")]
        command.creation_flags(CREATE_NO_WINDOW);

        command.output()
    });

    let cmd_display = format!("{} {:?}", cmd, args);

    let result = match tokio::time::timeout(timeout, task).await {
        Ok(join_result) => join_result.map_err(|e| SyncError::CommandExecutionFailed {
            command: cmd_display.clone(),
            reason: e.to_string(),
        })?,
        Err(_) => {
            tracing::error!(
                "[auto_installer] Command timed out after {:?}: {}",
                timeout,
                cmd_display
            );
            return Err(SyncError::CommandExecutionFailed {
                command: cmd_display,
                reason: format!("Command timed out after {} seconds", timeout.as_secs()),
            });
        }
    };

    let output = result.map_err(|e| SyncError::CommandExecutionFailed {
        command: cmd_display.clone(),
        reason: e.to_string(),
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("[auto_installer] Command failed: {}", stderr);
        return Err(SyncError::CommandExecutionFailed {
            command: cmd_display,
            reason: stderr.to_string(),
        });
    }

    Ok(())
}

/// Windows: 下载便携版Git（无需安装权限）
#[cfg(target_os = "windows")]
async fn download_portable_git() -> Result<()> {
    use std::fs;
    use std::path::PathBuf;

    tracing::info!("[auto_installer] Downloading portable Git...");

    let home = dirs::home_dir().ok_or(SyncError::HomeDirectoryNotFound)?;
    let portable_dir = home.join(".hajimi").join("portable");
    let git_dir = portable_dir.join("git");

    fs::create_dir_all(&git_dir).map_err(|e| SyncError::DirectoryCreationFailed {
        path: git_dir.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;

    // 下载MinGit（最小化Git）
    let url = "https://github.com/git-for-windows/git/releases/download/v2.43.0.windows.1/MinGit-2.43.0-64-bit.zip";

    tracing::info!("[auto_installer] Downloading from {}", url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| SyncError::Other(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| SyncError::Other(format!("Download failed: {}", e)))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| SyncError::Other(format!("Download failed: {}", e)))?;

    let zip_path = git_dir.join("mingit.zip");
    fs::write(&zip_path, &bytes).map_err(|e| SyncError::FileWriteFailed {
        path: zip_path.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;

    // 解压
    tracing::info!("[auto_installer] Extracting Git...");
    extract_zip(&zip_path, &git_dir)?;

    // 添加到PATH（仅本进程）
    let git_bin = git_dir.join("cmd");
    if let Ok(mut path) = std::env::var("PATH") {
        path.push_str(";");
        path.push_str(&git_bin.to_string_lossy());
        std::env::set_var("PATH", path);
    }

    tracing::info!("[auto_installer] Portable Git installed successfully");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
async fn download_portable_git() -> Result<()> {
    Err(SyncError::Other(
        "Portable Git only available on Windows".to_string(),
    ))
}

/// 解压ZIP文件
#[cfg(target_os = "windows")]
fn extract_zip(zip_path: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    use std::fs::File;
    use zip::ZipArchive;

    let file = File::open(zip_path).map_err(|e| SyncError::FileReadFailed {
        path: zip_path.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| SyncError::Other(e.to_string()))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| SyncError::Other(e.to_string()))?;
        let outpath = dest.join(file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| SyncError::DirectoryCreationFailed {
                path: outpath.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    SyncError::DirectoryCreationFailed {
                        path: parent.to_string_lossy().to_string(),
                        reason: e.to_string(),
                    }
                })?;
            }
            let mut outfile = File::create(&outpath).map_err(|e| SyncError::FileWriteFailed {
                path: outpath.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| SyncError::Other(e.to_string()))?;
        }
    }

    Ok(())
}

/// 安装独立版Node.js（无需包管理器）
async fn install_nodejs_standalone() -> Result<()> {
    tracing::info!("[auto_installer] Installing standalone Node.js...");

    let home = dirs::home_dir().ok_or(SyncError::HomeDirectoryNotFound)?;
    let node_dir = home.join(".hajimi").join("nodejs");

    #[cfg(target_os = "windows")]
    let url = "https://nodejs.org/dist/v22.16.0/node-v22.16.0-win-x64.zip";

    #[cfg(target_os = "macos")]
    let url = if cfg!(target_arch = "aarch64") {
        "https://nodejs.org/dist/v22.16.0/node-v22.16.0-darwin-arm64.tar.gz"
    } else {
        "https://nodejs.org/dist/v22.16.0/node-v22.16.0-darwin-x64.tar.gz"
    };

    #[cfg(target_os = "linux")]
    let url = "https://nodejs.org/dist/v22.16.0/node-v22.16.0-linux-x64.tar.xz";

    download_and_extract(url, &node_dir).await?;

    // 添加到PATH
    let bin_dir = node_dir.join("bin");
    add_to_path(&bin_dir)?;

    Ok(())
}

/// Linux: 使用NodeSource安装Node.js
#[cfg(target_os = "linux")]
async fn install_nodejs_nodesource() -> Result<()> {
    // 下载并执行NodeSource安装脚本
    let script = "curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -";
    run_silent_command("sh", &["-c", script]).await?;

    // 安装Node.js
    if check_command_exists("apt-get") {
        run_silent_command("sudo", &["apt-get", "install", "-y", "nodejs"]).await?;
    } else {
        run_silent_command("sudo", &["yum", "install", "-y", "nodejs"]).await?;
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
async fn install_nodejs_nodesource() -> Result<()> {
    Ok(())
}

/// 下载并解压文件
async fn download_and_extract(url: &str, dest: &std::path::Path) -> Result<()> {
    use std::fs;

    tracing::info!("[auto_installer] Downloading from {}", url);

    fs::create_dir_all(dest).map_err(|e| SyncError::DirectoryCreationFailed {
        path: dest.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| SyncError::Other(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| SyncError::Other(format!("Download failed: {}", e)))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| SyncError::Other(format!("Download failed: {}", e)))?;

    let temp_file = dest.join("download.tmp");
    fs::write(&temp_file, &bytes).map_err(|e| SyncError::FileWriteFailed {
        path: temp_file.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;

    // 根据文件扩展名解压
    if url.ends_with(".zip") {
        #[cfg(target_os = "windows")]
        extract_zip(&temp_file, dest)?;
    } else if url.ends_with(".tar.gz") || url.ends_with(".tar.xz") {
        extract_tar(&temp_file, dest)?;
    }

    fs::remove_file(&temp_file).ok();

    Ok(())
}

/// 解压tar文件
#[cfg(not(target_os = "windows"))]
fn extract_tar(archive_path: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    use std::process::Command;

    let extension = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let flag = if extension == "xz" { "xf" } else { "xzf" };

    Command::new("tar")
        .arg(flag)
        .arg(archive_path)
        .arg("-C")
        .arg(dest)
        .arg("--strip-components=1")
        .output()
        .map_err(|e| SyncError::CommandExecutionFailed {
            command: "tar".to_string(),
            reason: e.to_string(),
        })?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn extract_tar(_archive_path: &std::path::Path, _dest: &std::path::Path) -> Result<()> {
    Ok(())
}

/// 添加目录到PATH（仅本进程）
fn add_to_path(dir: &std::path::Path) -> Result<()> {
    if let Ok(mut path) = std::env::var("PATH") {
        let separator = if cfg!(windows) { ";" } else { ":" };
        path.push_str(separator);
        path.push_str(&dir.to_string_lossy());
        std::env::set_var("PATH", path);
        tracing::info!("[auto_installer] Added to PATH: {:?}", dir);
    }
    Ok(())
}

/// Tauri command: 自动安装所有缺失依赖
#[tauri::command]
pub async fn auto_install_dependencies() -> std::result::Result<Vec<InstallProgress>, String> {
    let mut results = Vec::new();

    // 检测并安装Git
    if !check_command_exists("git") {
        results.push(InstallProgress {
            tool: "git".to_string(),
            status: InstallStatus::Installing,
            progress: 0,
            message: "Installing Git...".to_string(),
        });

        match auto_install_git().await {
            Ok(_) => {
                results.push(InstallProgress {
                    tool: "git".to_string(),
                    status: InstallStatus::Completed,
                    progress: 100,
                    message: "Git installed successfully".to_string(),
                });
            }
            Err(e) => {
                tracing::error!("[auto_install] Git installation failed: {}", e);
                results.push(InstallProgress {
                    tool: "git".to_string(),
                    status: InstallStatus::Failed,
                    progress: 0,
                    message: format!("Failed: {}", e),
                });
            }
        }
    } else {
        results.push(InstallProgress {
            tool: "git".to_string(),
            status: InstallStatus::Skipped,
            progress: 100,
            message: "Already installed".to_string(),
        });
    }

    // 检测并安装Node.js
    if !check_command_exists("node") {
        results.push(InstallProgress {
            tool: "nodejs".to_string(),
            status: InstallStatus::Installing,
            progress: 0,
            message: "Installing Node.js...".to_string(),
        });

        match auto_install_nodejs().await {
            Ok(_) => {
                results.push(InstallProgress {
                    tool: "nodejs".to_string(),
                    status: InstallStatus::Completed,
                    progress: 100,
                    message: "Node.js installed successfully".to_string(),
                });
            }
            Err(e) => {
                tracing::error!("[auto_install] Node.js installation failed: {}", e);
                results.push(InstallProgress {
                    tool: "nodejs".to_string(),
                    status: InstallStatus::Failed,
                    progress: 0,
                    message: format!("Failed: {}", e),
                });
            }
        }
    } else {
        results.push(InstallProgress {
            tool: "nodejs".to_string(),
            status: InstallStatus::Skipped,
            progress: 100,
            message: "Already installed".to_string(),
        });
    }

    Ok(results)
}

/// Tauri command: 安装特定CLI工具
#[tauri::command]
pub async fn install_cli_tool(tool: String) -> std::result::Result<InstallProgress, String> {
    // Use enhanced detection (same as get_all_cli_status) to avoid false negatives
    if utils::resolve_executable(&tool).is_some() || check_command_exists(&tool) {
        return Ok(InstallProgress {
            tool: tool.clone(),
            status: InstallStatus::Skipped,
            progress: 100,
            message: "Already installed".to_string(),
        });
    }

    match auto_install_cli_tool(&tool).await {
        Ok(_) => Ok(InstallProgress {
            tool: tool.clone(),
            status: InstallStatus::Completed,
            progress: 100,
            message: format!("{} installed successfully", tool),
        }),
        Err(e) => Ok(InstallProgress {
            tool: tool.clone(),
            status: InstallStatus::Failed,
            progress: 0,
            message: format!("Failed: {}", e),
        }),
    }
}
