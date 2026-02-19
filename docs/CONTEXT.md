# Project Context — hajimi-cli-sync

## Overview
A Tauri v2 desktop app that syncs API proxy configurations (base URL + API key + model) to multiple AI CLI tools with one click. Built with React 19 + TypeScript frontend and Rust backend.

## Tech Stack
- **Frontend**: React 19, TypeScript, Vite, Tailwind CSS 4, DaisyUI 5, i18next
- **Backend**: Rust (Tauri v2), reqwest, serde, toml/toml_edit, tokio
- **Build**: `npm run tauri:dev` / `npm run tauri:build`

## Architecture

### Backend (`src-tauri/src/`)
| File | Purpose |
|---|---|
| `lib.rs` | Tauri commands: `get_all_cli_status`, `sync_cli`, `sync_all`, `restore_cli`, `fetch_models`, `test_connection`, `get_config_content` |
| `cli_sync.rs` | Core sync logic for Claude, Codex, Gemini (env-file based CLIs) |
| `opencode_sync.rs` | OpenCode JSON config sync |
| `droid_sync.rs` | Droid (Android Studio AI) settings.json sync |
| `auto_installer.rs` | Auto-detect & install missing CLI tools |
| `system_check.rs` | System requirements validation |
| `utils.rs` | URL validation, file helpers |
| `error.rs` | Error types (thiserror) |

### Frontend (`src/`)
| File | Purpose |
|---|---|
| `App.tsx` | Main app component |
| `types.ts` | TypeScript type definitions |
| `i18n.ts` | i18next configuration |
| `locales/` | Translation files |
| `components/` | UI components |
| `hooks/` | Custom React hooks |

### Supported CLI Tools (current)
1. **Claude** — `~/.claude/.env` (ANTHROPIC_BASE_URL, ANTHROPIC_API_KEY)
2. **Codex** — env-based config
3. **Gemini** — env-based config
4. **OpenCode** — `opencode.json`
5. **Droid** — `settings.json` (multi-provider)

## Current Progress

### Completed (P0/P1)
- Core sync for Claude, Codex, Gemini, OpenCode, Droid
- Auto-installer for missing CLI dependencies
- `syncAll` with per-CLI model override support
- Connection test and model fetching
- Bug fixes: Claude restore empty key, Droid parse crash, auto_installer npm name, syncAll ignoring perCliModels
- Single CLI restore without confirmation + env newline + connection test
- Core backend tests

### In Progress
- **Task #19**: Create `extra_clients.rs` with 10 new AI client support

### Pending
- **Task #20**: Update `lib.rs` to integrate extra clients
- **Task #21**: Update frontend `types.ts` and i18n for new clients
- **Task #22**: Update `auto_installer.rs` for new client installs
- **Task #23**: Verify compilation and run tests
- **Task #18**: P2 — Extract ModelSelector + unified errors + empty state + version number

## Key Patterns
- Each CLI tool has its own sync module with `check_*_installed()`, `get_sync_status()`, `sync_*_config()`, `restore_*_config()` functions
- `get_proxy_url()` in `lib.rs` handles URL format differences (e.g., codex/opencode need `/v1` suffix)
- `sync_all` iterates all installed CLIs, supports per-CLI model overrides via `per_cli_models` HashMap
- Backup/restore pattern: backup original config before sync, restore from backup
