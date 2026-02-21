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

export interface SwitchResult {
  success: boolean;
  errors: SyncResult[];
}

export interface AppConfig {
  url: string;
  apiKey: string;
  defaultModel: string;
}

export interface ProviderRecord {
  id: string;
  name: string;
  url: string;
  api_key: string;
  default_model: string;
  per_cli_models: string; // JSON string: Record<string, string>
  is_current: boolean;
  sort_index: number | null;
  notes: string | null;
  created_at: number;
}

export type InstallType = "npm" | "vscode" | "desktop" | "manual" | "manual-config";
export type CliCategory = "coding" | "chat" | "agent" | "rp";

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
  /** macOS app name for launching via `open -a`, e.g. "Cherry Studio" */
  launchName?: string;
  /** Deep link URL template for one-click provider import, e.g. "cherrystudio://providers/api-keys?v=1&data={config}" */
  deepLinkTemplate?: string;
  /** Web-browsable community/marketplace URL for external link button */
  communityUrl?: string;
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
  session_cookie: string | null;
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
  { id: "claude", name: "Claude Code", icon: "terminal", color: "border-purple-400", installType: "npm", category: "coding", descKey: "toolDesc.claude", postSyncHintKey: "toolHint.claude" },
  { id: "claude-vscode", name: "Claude Code (VS Code)", icon: "file-code", color: "border-purple-300", installType: "vscode", category: "coding", descKey: "toolDesc.claudeVscode", communityUrl: "https://marketplace.visualstudio.com/items?itemName=anthropic.claude-code" },
  { id: "opencode", name: "OpenCode", icon: "file-code", color: "border-orange-400", installType: "manual", category: "coding", downloadUrl: "https://github.com/anomalyco/opencode", descKey: "toolDesc.opencode" },
  { id: "codex", name: "Codex AI", icon: "code", color: "border-blue-400", installType: "npm", category: "coding", descKey: "toolDesc.codex" },
  { id: "gemini", name: "Gemini CLI", icon: "sparkles", color: "border-green-400", installType: "npm", category: "coding", descKey: "toolDesc.gemini" },
  { id: "droid", name: "Droid", icon: "bot", color: "border-red-400", installType: "desktop", category: "coding", downloadUrl: "https://factory.ai", descKey: "toolDesc.droid", launchName: "Droid" },
  { id: "cline", name: "Cline", icon: "file-text", color: "border-teal-400", installType: "manual-config", category: "coding", downloadUrl: "vscode:extension/saoudrizwan.claude-dev", descKey: "toolDesc.cline", postSyncHintKey: "toolHint.cline", communityUrl: "https://marketplace.visualstudio.com/items?itemName=saoudrizwan.claude-dev" },
  { id: "roo-code", name: "Roo Code", icon: "rabbit", color: "border-amber-400", installType: "manual-config", category: "coding", downloadUrl: "vscode:extension/rooveterinaryinc.roo-cline", descKey: "toolDesc.rooCode", postSyncHintKey: "toolHint.rooCode", communityUrl: "https://marketplace.visualstudio.com/items?itemName=RooVeterinaryInc.roo-cline" },
  { id: "kilo-code", name: "Kilo Code", icon: "ruler", color: "border-lime-400", installType: "manual-config", category: "coding", downloadUrl: "vscode:extension/kilocode.kilo-code", descKey: "toolDesc.kiloCode", postSyncHintKey: "toolHint.kiloCode", communityUrl: "https://marketplace.visualstudio.com/items?itemName=kilocode.kilo-code" },
  { id: "cursor", name: "Cursor", icon: "mouse-pointer", color: "border-cyan-400", installType: "manual-config", category: "coding", downloadUrl: "https://cursor.com/downloads", descKey: "toolDesc.cursor", postSyncHintKey: "toolHint.cursor", launchName: "Cursor" },
  { id: "chatbox", name: "Chatbox", icon: "message-square", color: "border-sky-400", installType: "desktop", category: "chat", downloadUrl: "https://chatboxai.app", descKey: "toolDesc.chatbox", launchName: "Chatbox" },
  { id: "cherry-studio", name: "Cherry Studio", icon: "cherry", color: "border-pink-400", installType: "desktop", category: "chat", downloadUrl: "https://cherry-ai.com", descKey: "toolDesc.cherryStudio", launchName: "Cherry Studio", deepLinkTemplate: "cherrystudio://providers/api-keys?v=1&data={config}" },
  { id: "jan", name: "Jan", icon: "cpu", color: "border-indigo-400", installType: "desktop", category: "chat", downloadUrl: "https://jan.ai/download", descKey: "toolDesc.jan", launchName: "Jan" },
  { id: "lobechat", name: "LobeChat", icon: "brain", color: "border-violet-400", installType: "manual-config", category: "chat", downloadUrl: "https://lobehub.com/zh", descKey: "toolDesc.lobechat", postSyncHintKey: "toolHint.lobechat", launchName: "LobeChat" },
  { id: "boltai", name: "BoltAI", icon: "zap", color: "border-slate-400", installType: "manual-config", category: "chat", downloadUrl: "https://boltai.com", descKey: "toolDesc.boltai", postSyncHintKey: "toolHint.boltai", launchName: "BoltAI" },
  { id: "openclaw", name: "OpenClaw", icon: "waves", color: "border-rose-400", installType: "npm", category: "agent", downloadUrl: "https://docs.openclaw.ai", descKey: "toolDesc.openclaw", postSyncHintKey: "toolHint.openclaw" },
  { id: "sillytavern", name: "SillyTavern", icon: "beer", color: "border-yellow-400", installType: "manual", category: "rp", downloadUrl: "https://docs.sillytavern.app/installation/", descKey: "toolDesc.sillytavern", postSyncHintKey: "toolHint.sillytavern" },
];

