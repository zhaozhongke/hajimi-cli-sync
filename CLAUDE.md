# CLAUDE.md — hajimi-cli-sync

## Project
Tauri v2 desktop app that syncs API proxy configs to multiple AI CLI tools. React 19 + Rust backend.

## Quick Start
```bash
npm run tauri:dev    # Dev mode
npm run tauri:build  # Production build
```

## Key Commands
```bash
cd src-tauri && cargo check   # Rust type check
cd src-tauri && cargo test    # Run backend tests
npm run build                 # Frontend build
```

## Code Conventions
- Rust modules: one file per CLI tool (`cli_sync.rs`, `opencode_sync.rs`, `droid_sync.rs`)
- Each module exports: `check_*_installed()`, `get_sync_status()`, `sync_*_config()`, `restore_*_config()`
- Tauri commands in `lib.rs` dispatch to modules via `match app.as_str()`
- Error handling: `thiserror` in backend, string errors at Tauri command boundary
- Frontend i18n: `src/locales/` with i18next
- UI: DaisyUI 5 components with Tailwind CSS 4

## HOF Protocol
When `HOF` is invoked:
1. Update `docs/CONTEXT.md` with current architecture and task state
2. Update `docs/PROGRESS.md` with session summary
3. Commit: `git add docs/ && git commit -m "chore: HOF handoff update"`

## Important Notes
- `get_proxy_url()` adds `/v1` suffix for codex/opencode only
- `sync_all` skips uninstalled CLIs silently
- Backup files use `.bak` extension pattern
- Config files may contain sensitive API keys — never log values
