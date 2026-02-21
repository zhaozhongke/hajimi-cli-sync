import { invoke } from "@tauri-apps/api/core";
import { useState, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import type { CliStatusResult, SyncAllResult } from "../types";

export interface SyncLogEntry {
  id: number;
  time: string;
  action: "sync" | "sync_all" | "restore" | "install";
  app: string;
  success: boolean;
  detail?: string;
}

const LOG_KEY = "hajimi-sync-log";
const MAX_LOG_ENTRIES = 50;

function readLog(): SyncLogEntry[] {
  try {
    const raw = localStorage.getItem(LOG_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function appendLog(entry: Omit<SyncLogEntry, "id" | "time">): SyncLogEntry {
  const log = readLog();
  const newEntry: SyncLogEntry = {
    ...entry,
    id: Date.now(),
    time: new Date().toISOString(),
  };
  log.unshift(newEntry);
  if (log.length > MAX_LOG_ENTRIES) log.length = MAX_LOG_ENTRIES;
  localStorage.setItem(LOG_KEY, JSON.stringify(log));
  return newEntry;
}

export function useCliSync() {
  const { t } = useTranslation();
  const [statuses, setStatuses] = useState<CliStatusResult[]>([]);
  const statusesRef = useRef<CliStatusResult[]>(statuses);
  statusesRef.current = statuses;
  const [loading, setLoading] = useState(false);
  const [syncing, setSyncing] = useState<Record<string, boolean>>({});
  const [restoring, setRestoring] = useState<Record<string, boolean>>({});
  const [installing, setInstalling] = useState<Record<string, boolean>>({});

  const detectAll = useCallback(async (url: string) => {
    setLoading(true);
    try {
      const result = await invoke<CliStatusResult[]>("get_all_cli_status", {
        url,
      });
      setStatuses(result);
    } catch (e) {
      console.error("Failed to detect CLIs:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const syncOne = useCallback(
    async (
      app: string,
      url: string,
      apiKey: string,
      model: string | null,
      name: string
    ) => {
      setSyncing((prev) => ({ ...prev, [app]: true }));
      try {
        await invoke("sync_cli", { app, url, apiKey, model });
        toast.success(t("toast.syncSuccess", { name }));
        appendLog({ action: "sync", app: name, success: true });
        const allStatus = await invoke<CliStatusResult[]>(
          "get_all_cli_status",
          { url }
        );
        setStatuses(allStatus);
      } catch (e: unknown) {
        const error = e instanceof Error ? e.message : String(e);
        toast.error(t("toast.syncFailed", { name, error }), { duration: 5000 });
        appendLog({ action: "sync", app: name, success: false, detail: error });
      } finally {
        setSyncing((prev) => ({ ...prev, [app]: false }));
      }
    },
    [t]
  );

  const syncAll = useCallback(
    async (url: string, apiKey: string, model: string | null, perCliModels?: Record<string, string>) => {
      setSyncing((prev) => {
        const next = { ...prev };
        statusesRef.current
          .filter((s) => s.installed)
          .forEach((s) => (next[s.app] = true));
        return next;
      });
      try {
        const result = await invoke<SyncAllResult>("sync_all", {
          url,
          apiKey,
          model,
          perCliModels: perCliModels || null,
        });
        const successCount = result.results.filter((r) => r.success).length;
        const totalCount = result.results.length;
        if (successCount === totalCount && totalCount > 0) {
          toast.success(
            t("toast.syncAllSuccess", { success: successCount, total: totalCount })
          );
          appendLog({ action: "sync_all", app: `${successCount}/${totalCount}`, success: true });
        } else if (totalCount === 0) {
          toast.error(t("toast.noInstalledCli"), { duration: 5000 });
        } else {
          toast.error(t("toast.syncAllFailed"), { duration: 5000 });
          appendLog({ action: "sync_all", app: `${successCount}/${totalCount}`, success: false });
        }
        const allStatus = await invoke<CliStatusResult[]>(
          "get_all_cli_status",
          { url }
        );
        setStatuses(allStatus);
      } catch (e: unknown) {
        const error = e instanceof Error ? e.message : String(e);
        toast.error(t("toast.syncFailed", { name: "Sync All", error }), { duration: 5000 });
      } finally {
        setSyncing({});
      }
    },
    [t]
  );

  const restoreOne = useCallback(
    async (app: string, url: string, name: string) => {
      setRestoring((prev) => ({ ...prev, [app]: true }));
      try {
        await invoke("restore_cli", { app });
        toast.success(t("toast.restoreSuccess", { name }));
        appendLog({ action: "restore", app: name, success: true });
        const allStatus = await invoke<CliStatusResult[]>(
          "get_all_cli_status",
          { url }
        );
        setStatuses(allStatus);
      } catch (e: unknown) {
        const error = e instanceof Error ? e.message : String(e);
        toast.error(t("toast.restoreFailed", { name, error }), { duration: 5000 });
        appendLog({ action: "restore", app: name, success: false, detail: error });
      } finally {
        setRestoring((prev) => ({ ...prev, [app]: false }));
      }
    },
    [t]
  );

  const installOne = useCallback(
    async (app: string, url: string, name: string, downloadUrl?: string) => {
      setInstalling((prev) => ({ ...prev, [app]: true }));
      const handleFail = (error: string) => {
        if (downloadUrl) {
          toast.info(t("install.failedOpenDownload", { name }));
          invoke("open_external_url", { url: downloadUrl });
        } else {
          toast.error(t("install.failed", { error }));
        }
      };
      try {
        const result = await invoke<{
          tool: string;
          status: string;
          progress: number;
          message: string;
        }>("install_cli_tool", { tool: app });
        if (result.status === "failed") {
          handleFail(result.message);
          appendLog({ action: "install", app: name, success: false, detail: result.message });
        } else {
          toast.success(t("install.success", { name }));
          appendLog({ action: "install", app: name, success: true });
          // Wait for PATH to refresh before re-detecting
          await new Promise((r) => setTimeout(r, 2000));
          const allStatus = await invoke<CliStatusResult[]>(
            "get_all_cli_status",
            { url }
          );
          setStatuses(allStatus);
        }
      } catch (e: unknown) {
        const error = e instanceof Error ? e.message : String(e);
        handleFail(error);
      } finally {
        setInstalling((prev) => ({ ...prev, [app]: false }));
      }
    },
    [t]
  );

  const getConfigContent = useCallback(
    async (app: string, fileName?: string): Promise<string> => {
      try {
        return await invoke<string>("get_config_content", {
          app,
          fileName: fileName || null,
        });
      } catch (e: unknown) {
        const error = e instanceof Error ? e.message : String(e);
        return `Error: ${error}`;
      }
    },
    []
  );

  return {
    statuses,
    loading,
    syncing,
    restoring,
    installing,
    detectAll,
    syncOne,
    syncAll,
    restoreOne,
    installOne,
    getConfigContent,
  };
}

export { readLog as getSyncLog };
