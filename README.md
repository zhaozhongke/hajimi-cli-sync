# hajimi-cli-sync

<p align="center">
  <img src="./src-tauri/icons/128x128.png" alt="hajimi-cli-sync logo" width="128">
</p>

<p align="center">
  <a href="#english">English</a> | <a href="#简体中文">简体中文</a>
</p>

---

<h2 id="english">🇺🇸 English</h2>

A powerful, one-click desktop application built with Tauri v2 that seamlessly synchronizes API proxy configurations (Base URL, API Key, and Model) across multiple AI CLI tools and IDE extensions.

### ✨ Features

- **One-Click Sync**: Propagate your central API configuration to multiple supported AI tools simultaneously.
- **Auto-Detection**: Automatically detects which CLI tools are installed on your system.
- **Model Overrides**: Supports setting specific, distinct models for individual CLI tools while keeping the base API credentials synced.
- **Safe Modifications**: Implements a robust Backup & Restore mechanism. Original configurations are backed up before modifications, allowing you to easily roll back changes.
- **Connection Testing**: Built-in utility to verify API connectivity and fetch available models directly from the proxy before syncing.
- **Modern UI**: Clean, responsive interface built with React 19, Tailwind CSS v4, and DaisyUI v5. Supports Light and Dark modes.
- **Internationalization (i18n)**: Full English and Simplified Chinese (zh-CN) support.
- **Cross-Platform**: Fast and lightweight, available for macOS, Windows, and Linux (powered by Rust and Tauri).

### 🚀 Supported AI Clients

The application natively parses and updates the specific configuration files for the following tools:

**Core CLI Tools**
- **Claude CLI** (`~/.claude/.env`)
- **Codex** (`~/.codex/.env`)
- **Gemini CLI** (`~/.gemini/.env`)
- **OpenCode** (`opencode.json`)
- **Droid** (Android Studio AI, `settings.json`)

**Extra Clients & IDE Extensions (In Progress)**
- **Claude VSCode Extension**
- **Chatbox**
- **CherryStudio**
- **Jan**
- **Cursor**
- **Cline**
- **RooCode**
- **KiloCode**
- **SillyTavern**
- **LobeChat**
- **BoltAI**

### 📸 Screenshots

*(Add screenshots here using the images provided in the repository, e.g., `screenshot-current.png`, `screenshot-dark.png`)*

### 🛠️ Technology Stack

- **Frontend**: [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/) + [Tailwind CSS v4](https://tailwindcss.com/) + [DaisyUI v5](https://daisyui.com/) + i18next
- **Backend (Tauri)**: [Tauri v2](https://v2.tauri.app/) + [Rust](https://www.rust-lang.org/) (2021) + [Tokio](https://tokio.rs/) + `reqwest` + `toml` / `serde_json`

### 📦 Download & Install

#### macOS
Download the `.dmg` file from [Releases](https://github.com/hajimi-ai/hajimi-cli-sync/releases). Open the `.dmg` and drag the app to **Applications**.

#### Windows
Download the `.msi` or `.exe` installer from [Releases](https://github.com/hajimi-ai/hajimi-cli-sync/releases) and double-click to install.

#### Linux
Download `.deb` (Debian/Ubuntu) or `.AppImage` (universal) from [Releases](https://github.com/hajimi-ai/hajimi-cli-sync/releases).

### 🛠️ Troubleshooting

#### macOS: "app is damaged" or "cannot be opened"?

Due to macOS security (Gatekeeper), apps downloaded outside the App Store may trigger this warning. To fix it, open Terminal and run:

```bash
sudo xattr -rd com.apple.quarantine "/Applications/哈基米AI Switch.app"
```

> **If you get `option -r not recognized`:** Your terminal is using a Python-based `xattr` (installed via conda, pip, Homebrew, etc.) instead of the macOS system version. Use the full path to the system binary:
> ```bash
> sudo /usr/bin/xattr -rd com.apple.quarantine "/Applications/哈基米AI Switch.app"
> ```

---

### 🛠️ Development

**Prerequisites**: Node.js (v18+) and Rust (latest stable).

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd hajimi-cli-sync
   ```
2. **Install frontend dependencies:**
   ```bash
   npm install
   ```
3. **Start the development server:**
   ```bash
   npm run tauri:dev
   ```
4. **Build for production:**
   ```bash
   npm run tauri:build
   ```

### 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<h2 id="简体中文">🇨🇳 简体中文</h2>

一款基于 Tauri v2 构建的强大、支持一键操作的桌面应用程序。它可以将您的 API 代理配置（Base URL、API Key 和默认模型）无缝同步到系统中的多个 AI CLI 工具和 IDE 扩展中。

### ✨ 核心功能

- **一键同步**：将您的集中式 API 配置一次性分发、应用到所有受支持的 AI 工具中。
- **自动检测**：自动扫描并检测您的系统上已安装了哪些受支持的 CLI/IDE 工具。
- **模型独立覆盖**：支持为每个工具单独设置不同的模型（例如 Claude CLI 用 `claude-3-5-sonnet`，OpenCode 用 `gpt-4o`），同时保持底层的 API 密钥与地址同步。
- **安全的修改机制**：内置可靠的“备份与还原”机制。在修改任何配置之前都会自动备份原文件，允许您随时一键回滚。
- **连接性测试**：内置网络测试工具，在同步前可验证 API 的连通性，并直接从代理服务端拉取支持的模型列表。
- **现代化 UI**：采用 React 19、Tailwind CSS v4 和 DaisyUI v5 构建的响应式界面，并提供明暗模式切换。
- **国际化 (i18n)**：应用内原生支持简体中文和英文。
- **跨平台支持**：得益于 Rust 和 Tauri 底层架构，应用极度轻量且运行迅速，全面支持 macOS、Windows 和 Linux。

### 🚀 支持的 AI 客户端

本应用原生解析并更新以下工具的本地配置文件：

**核心 CLI 工具**
- **Claude CLI** (`~/.claude/.env`)
- **Codex** (`~/.codex/.env`)
- **Gemini CLI** (`~/.gemini/.env`)
- **OpenCode** (`opencode.json`)
- **Droid** (Android Studio AI, `settings.json`)

**扩展客户端与 IDE 插件（开发中 / In Progress）**
- **Claude VSCode Extension**
- **Chatbox**
- **CherryStudio**
- **Jan**
- **Cursor**
- **Cline**
- **RooCode**
- **KiloCode**
- **SillyTavern**
- **LobeChat**
- **BoltAI**

### 📸 界面截图

*(请在此处添加您的截图文件，例如 `screenshot-current.png`, `screenshot-dark.png`)*

### 🛠️ 技术栈

- **前端框架**：[React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/) + [Tailwind CSS v4](https://tailwindcss.com/) + [DaisyUI v5](https://daisyui.com/) + i18next
- **后端 (Tauri)**：[Tauri v2](https://v2.tauri.app/) + [Rust](https://www.rust-lang.org/) (Edition 2021) + [Tokio](https://tokio.rs/) 异步运行时 + `reqwest` + `toml` / `serde_json` 解析器

### 📦 下载安装

#### macOS
从 [Releases](https://github.com/hajimi-ai/hajimi-cli-sync/releases) 下载 `.dmg` 文件，打开后将应用拖入 **Applications（应用程序）** 文件夹。

#### Windows
从 [Releases](https://github.com/hajimi-ai/hajimi-cli-sync/releases) 下载 `.msi` 或 `.exe` 安装包，双击安装即可。

#### Linux
从 [Releases](https://github.com/hajimi-ai/hajimi-cli-sync/releases) 下载 `.deb`（Debian/Ubuntu）或 `.AppImage`（通用格式）安装。

### 🛠️ 常见问题排查 (Troubleshooting)

#### macOS 提示"应用已损坏，无法打开"？

由于 macOS 的安全机制（Gatekeeper），非 App Store 下载的应用可能会触发此提示。打开终端，执行以下命令即可修复：

```bash
sudo xattr -rd com.apple.quarantine "/Applications/哈基米AI Switch.app"
```

> **如果报错 `option -r not recognized`：** 说明您的终端调用的是通过 conda、pip、Homebrew 等安装的 Python 版 `xattr`，而非 macOS 系统自带版本。请使用系统版本的完整路径：
> ```bash
> sudo /usr/bin/xattr -rd com.apple.quarantine "/Applications/哈基米AI Switch.app"
> ```

---

### 🛠️ 本地开发

**环境要求**：Node.js (v18 或更高版本) 以及 Rust (最新稳定版)。

1. **克隆仓库：**
   ```bash
   git clone <repository-url>
   cd hajimi-cli-sync
   ```
2. **安装前端依赖：**
   ```bash
   npm install
   ```
3. **启动开发服务器：**
   ```bash
   # 这将同时启动 Vite 服务和 Tauri Rust 独立窗口
   npm run tauri:dev
   ```
4. **编译打包构建：**
   ```bash
   npm run tauri:build
   ```

### 📄 开源协议

本项目采用 MIT 开源许可证。详见 [LICENSE](LICENSE) 文件。
