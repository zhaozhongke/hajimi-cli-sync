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

export interface CliInfo {
  id: string;
  name: string;
  icon: string;
  color: string;
}

export const CLI_LIST: CliInfo[] = [
  { id: "claude", name: "Claude Code", icon: "🟣", color: "border-purple-400" },
  { id: "codex", name: "Codex AI", icon: "🔵", color: "border-blue-400" },
  { id: "gemini", name: "Gemini CLI", icon: "🟢", color: "border-green-400" },
  { id: "opencode", name: "OpenCode", icon: "🟠", color: "border-orange-400" },
  { id: "droid", name: "Droid", icon: "🔴", color: "border-red-400" },
];

export interface ModelGroup {
  label: string;
  models: string[];
}

export const MODEL_GROUPS: ModelGroup[] = [
  {
    label: "Claude",
    models: [
      "claude-sonnet-4-5",
      "claude-sonnet-4-5-thinking",
      "claude-opus-4-5-thinking",
    ],
  },
  {
    label: "Gemini",
    models: [
      "gemini-3-pro-high",
      "gemini-3-pro-low",
      "gemini-3-flash",
      "gemini-2.5-flash",
      "gemini-2.5-pro",
    ],
  },
  {
    label: "OpenAI",
    models: ["gpt-4o", "o3"],
  },
];
