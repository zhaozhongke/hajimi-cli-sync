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
}

export const CLI_LIST: CliInfo[] = [
  { id: "claude", name: "Claude Code", icon: "terminal", color: "border-purple-400", installType: "npm", category: "cli" },
  { id: "opencode", name: "OpenCode", icon: "file-code", color: "border-orange-400", installType: "manual", category: "cli", downloadUrl: "https://github.com/anomalyco/opencode" },
  { id: "gemini", name: "Gemini CLI", icon: "sparkles", color: "border-green-400", installType: "npm", category: "cli" },
  { id: "openclaw", name: "OpenClaw", icon: "waves", color: "border-rose-400", installType: "manual", category: "cli", downloadUrl: "https://docs.openclaw.ai" },
  { id: "droid", name: "Droid", icon: "bot", color: "border-red-400", installType: "desktop", category: "desktop", downloadUrl: "https://factory.ai" },
  { id: "cursor", name: "Cursor", icon: "mouse-pointer", color: "border-cyan-400", installType: "manual-config", category: "desktop", downloadUrl: "https://cursor.com/downloads" },
  { id: "chatbox", name: "Chatbox", icon: "message-square", color: "border-sky-400", installType: "desktop", category: "desktop", downloadUrl: "https://chatboxai.app" },
  { id: "cherry-studio", name: "Cherry Studio", icon: "cherry", color: "border-pink-400", installType: "desktop", category: "desktop", downloadUrl: "https://cherry-ai.com" },
  { id: "jan", name: "Jan", icon: "cpu", color: "border-indigo-400", installType: "desktop", category: "desktop", downloadUrl: "https://jan.ai/download" },
  { id: "cline", name: "Cline", icon: "file-text", color: "border-teal-400", installType: "manual-config", category: "extension" },
  { id: "roo-code", name: "Roo Code", icon: "rabbit", color: "border-amber-400", installType: "manual-config", category: "extension" },
  { id: "kilo-code", name: "Kilo Code", icon: "ruler", color: "border-lime-400", installType: "manual-config", category: "extension" },
  { id: "sillytavern", name: "SillyTavern", icon: "beer", color: "border-yellow-400", installType: "manual", category: "desktop", downloadUrl: "https://docs.sillytavern.app/installation/" },
  { id: "lobechat", name: "LobeChat", icon: "brain", color: "border-violet-400", installType: "desktop", category: "desktop", downloadUrl: "https://lobehub.com/download" },
  { id: "boltai", name: "BoltAI", icon: "zap", color: "border-slate-400", installType: "desktop", category: "desktop", downloadUrl: "https://boltai.com" },
  { id: "codex", name: "Codex AI", icon: "code", color: "border-blue-400", installType: "npm", category: "cli" },
];

