import { invoke } from "@tauri-apps/api/core";
import type { ProviderRecord, SwitchResult } from "../types";

export async function listProviders(): Promise<ProviderRecord[]> {
  return invoke("list_providers");
}

export async function getCurrentProvider(): Promise<ProviderRecord | null> {
  return invoke("get_current_provider");
}

export async function saveProvider(provider: ProviderRecord): Promise<void> {
  return invoke("save_provider", { provider });
}

export async function deleteProvider(id: string): Promise<void> {
  return invoke("delete_provider", { id });
}

export async function switchProvider(id: string): Promise<SwitchResult> {
  return invoke("switch_provider", { id });
}

export async function reorderProviders(ids: string[]): Promise<void> {
  return invoke("reorder_providers", { ids });
}
