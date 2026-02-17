# 快速发布指南

## 🚀 自动构建多平台包

### 方式1：推送标签（推荐）

```bash
# 创建版本标签
git tag v1.0.0

# 推送到远程（自动触发构建）
git push origin v1.0.0
```

**自动生成的安装包**：
- ✅ Windows: `hajimi-cli-sync_1.0.0_x64.msi` + `setup.exe`
- ✅ macOS (Intel): `hajimi-cli-sync_1.0.0_x64.dmg`
- ✅ macOS (Apple Silicon): `hajimi-cli-sync_1.0.0_aarch64.dmg`
- ✅ Linux: `hajimi-cli-sync_1.0.0_amd64.deb` + `AppImage`

构建时间：约15-20分钟（并行构建所有平台）

---

### 方式2：手动触发

1. 访问 GitHub Actions 页面
2. 选择 "Build Multi-Platform Release"
3. 点击 "Run workflow"
4. 输入版本号（可选）
5. 等待构建完成

---

## 📦 构建矩阵

| 平台 | 架构 | 输出格式 | Runner |
|------|------|----------|--------|
| Windows | x64 | MSI + EXE | windows-latest |
| macOS | x64 | DMG + APP | macos-latest |
| macOS | ARM64 | DMG + APP | macos-latest |
| Linux | x64 | DEB + AppImage | ubuntu-22.04 |

---

## 🔐 配置签名（可选）

为了让应用通过操作系统安全检查，需要配置代码签名：

### Windows (可选)
```bash
# 在 GitHub Secrets 中添加
TAURI_SIGNING_PRIVATE_KEY
TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

### macOS (推荐)
```bash
# Apple Developer 证书
# 需要在 Tauri 配置中设置
```

---

## 📝 发布流程

### 1. 准备发布
```bash
# 更新版本号
npm version patch  # 或 minor/major

# 提交更改
git add .
git commit -m "chore: release v1.0.0"
git push
```

### 2. 创建标签
```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

### 3. 等待构建
- GitHub Actions 自动运行
- 并行构建所有平台
- 自动创建草稿 Release

### 4. 发布 Release
1. 访问 GitHub Releases 页面
2. 编辑草稿 Release
3. 添加更新日志
4. 点击 "Publish Release"

---

## 🎯 下载链接

发布后，用户可以从以下位置下载：

```
https://github.com/YOUR_USERNAME/hajimi-cli-sync/releases/latest
```

**各平台安装命令**：

```bash
# Windows (PowerShell)
winget install hajimi-cli-sync

# macOS
brew install --cask hajimi-cli-sync

# Linux (Debian/Ubuntu)
sudo dpkg -i hajimi-cli-sync_1.0.0_amd64.deb

# Linux (通用 AppImage)
chmod +x hajimi-cli-sync_1.0.0_amd64.AppImage
./hajimi-cli-sync_1.0.0_amd64.AppImage
```

---

## 🔄 自动发布到包管理器

Release 发布后，可以手动提交到各平台包管理器：

### Homebrew (macOS)
```bash
brew tap YOUR_USERNAME/tap
brew install hajimi-cli-sync
```

### Chocolatey (Windows)
```bash
choco install hajimi-cli-sync
```

### AUR (Arch Linux)
```bash
yay -S hajimi-cli-sync
```

---

## 🛠️ 调试构建失败

如果构建失败，检查：

1. **依赖问题**
   ```bash
   npm ci  # 清理并重新安装
   cargo clean
   ```

2. **Rust 版本**
   ```bash
   rustup update stable
   ```

3. **查看 Actions 日志**
   - GitHub → Actions → 点击失败的 workflow
   - 查看详细错误信息

---

## 📊 构建状态徽章

在 README.md 中添加：

```markdown
[![Build Status](https://github.com/YOUR_USERNAME/hajimi-cli-sync/workflows/Build%20Multi-Platform%20Release/badge.svg)](https://github.com/YOUR_USERNAME/hajimi-cli-sync/actions)
```
