export interface CliStatusResult {
  app: string;
  installed: boolean;
  version: string | null;
  is_synced: boolean;
  has_backup: boolean;
  current_base_url: string | null;
  files: string[];
  synced_count: number | null;
}

export interface SyncResult {
  app: string;
  success: boolean;
  error: string | null;
}

export interface SyncAllResult {
  results: SyncResult[];
}

export interface AppConfig {
  url: string;
  apiKey: string;
  defaultModel: string;
}

export type InstallType = "npm" | "vscode" | "desktop" | "manual" | "manual-config";
export type CliCategory = "cli" | "desktop" | "extension";

export interface CliInfo {
  id: string;
  name: string;
  icon: string;
  color: string;
  installType: InstallType;
  category: CliCategory;
  downloadUrl?: string;
  /** i18n key for one-line description, e.g. "toolDesc.claude" */
  descKey?: string;
  /** i18n key for post-sync next-step hint, e.g. "toolHint.openclaw" */
  postSyncHintKey?: string;
}

// ── Account login types ──

export interface PlatformInfo {
  system_name: string;
  version: string;
  register_enabled: boolean;
}

export interface AccountInfo {
  user_id: number;
  username: string;
  display_name: string;
}

export interface ApiTokenInfo {
  id: number;
  name: string;
  key: string;
  status: number; // 1=enabled, 2=disabled, 3=expired, 4=exhausted
  used_quota: number;
  remain_quota: number;
  unlimited_quota: boolean;
  expired_time: number; // unix timestamp, -1 = never
  model_limits_enabled: boolean;
  model_limits: string[];
}

export type AuthMode = "manual" | "account";

export const CLI_LIST: CliInfo[] = [
  { id: "claude", name: "Claude Code", icon: "terminal", color: "border-purple-400", installType: "npm", category: "cli", descKey: "toolDesc.claude" },
  { id: "opencode", name: "OpenCode", icon: "file-code", color: "border-orange-400", installType: "manual", category: "cli", downloadUrl: "https://github.com/anomalyco/opencode", descKey: "toolDesc.opencode" },
  { id: "codex", name: "Codex AI", icon: "code", color: "border-blue-400", installType: "npm", category: "cli", descKey: "toolDesc.codex" },
  { id: "gemini", name: "Gemini CLI", icon: "sparkles", color: "border-green-400", installType: "npm", category: "cli", descKey: "toolDesc.gemini" },
  { id: "openclaw", name: "OpenClaw", icon: "waves", color: "border-rose-400", installType: "npm", category: "cli", downloadUrl: "https://docs.openclaw.ai", descKey: "toolDesc.openclaw", postSyncHintKey: "toolHint.openclaw" },
  { id: "droid", name: "Droid", icon: "bot", color: "border-red-400", installType: "desktop", category: "desktop", downloadUrl: "https://factory.ai", descKey: "toolDesc.droid" },
  { id: "cursor", name: "Cursor", icon: "mouse-pointer", color: "border-cyan-400", installType: "manual-config", category: "desktop", downloadUrl: "https://cursor.com/downloads", descKey: "toolDesc.cursor", postSyncHintKey: "toolHint.manualConfig" },
  { id: "chatbox", name: "Chatbox", icon: "message-square", color: "border-sky-400", installType: "desktop", category: "desktop", downloadUrl: "https://chatboxai.app", descKey: "toolDesc.chatbox" },
  { id: "cherry-studio", name: "Cherry Studio", icon: "cherry", color: "border-pink-400", installType: "desktop", category: "desktop", downloadUrl: "https://cherry-ai.com", descKey: "toolDesc.cherryStudio" },
  { id: "jan", name: "Jan", icon: "cpu", color: "border-indigo-400", installType: "desktop", category: "desktop", downloadUrl: "https://jan.ai/download", descKey: "toolDesc.jan" },
  { id: "cline", name: "Cline", icon: "file-text", color: "border-teal-400", installType: "manual-config", category: "extension", descKey: "toolDesc.cline", postSyncHintKey: "toolHint.manualConfig" },
  { id: "roo-code", name: "Roo Code", icon: "rabbit", color: "border-amber-400", installType: "manual-config", category: "extension", descKey: "toolDesc.rooCode", postSyncHintKey: "toolHint.manualConfig" },
  { id: "kilo-code", name: "Kilo Code", icon: "ruler", color: "border-lime-400", installType: "manual-config", category: "extension", descKey: "toolDesc.kiloCode", postSyncHintKey: "toolHint.manualConfig" },
  { id: "sillytavern", name: "SillyTavern", icon: "beer", color: "border-yellow-400", installType: "manual", category: "desktop", downloadUrl: "https://docs.sillytavern.app/installation/", descKey: "toolDesc.sillytavern", postSyncHintKey: "toolHint.sillytavern" },
  { id: "lobechat", name: "LobeChat", icon: "brain", color: "border-violet-400", installType: "desktop", category: "desktop", downloadUrl: "https://lobehub.com/download", descKey: "toolDesc.lobechat" },
  { id: "boltai", name: "BoltAI", icon: "zap", color: "border-slate-400", installType: "desktop", category: "desktop", downloadUrl: "https://boltai.com", descKey: "toolDesc.boltai" },
];

