import { invoke } from "@tauri-apps/api/core";
import { useState, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import type { CliStatusResult, SyncAllResult } from "../types";

export function useCliSync() {
  const { t } = useTranslation();
  const [statuses, setStatuses] = useState<CliStatusResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [syncing, setSyncing] = useState<Record<string, boolean>>({});
  const [restoring, setRestoring] = useState<Record<string, boolean>>({});
  const [toasts, setToasts] = useState<
    { id: number; msg: string; type: "success" | "error" }[]
  >([]);
  const toastId = useRef(0);

  const addToast = useCallback(
    (msg: string, type: "success" | "error") => {
      const id = ++toastId.current;
      setToasts((prev) => [...prev, { id, msg, type }]);
      setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
      }, 3000);
    },
    []
  );

  const removeToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

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
        addToast(t("toast.syncSuccess", { name }), "success");
        const allStatus = await invoke<CliStatusResult[]>(
          "get_all_cli_status",
          { url }
        );
        setStatuses(allStatus);
      } catch (e: unknown) {
        const error = e instanceof Error ? e.message : String(e);
        addToast(t("toast.syncFailed", { name, error }), "error");
      } finally {
        setSyncing((prev) => ({ ...prev, [app]: false }));
      }
    },
    [addToast, t]
  );

  const syncAll = useCallback(
    async (url: string, apiKey: string, model: string | null) => {
      setSyncing((prev) => {
        const next = { ...prev };
        statuses
          .filter((s) => s.installed)
          .forEach((s) => (next[s.app] = true));
        return next;
      });
      try {
        const result = await invoke<SyncAllResult>("sync_all", {
          url,
          apiKey,
          model,
        });
        const successCount = result.results.filter((r) => r.success).length;
        const totalCount = result.results.length;
        if (successCount === totalCount && totalCount > 0) {
          addToast(
            t("toast.syncAllSuccess", { success: successCount, total: totalCount }),
            "success"
          );
        } else if (totalCount === 0) {
          addToast(t("toast.noInstalledCli"), "error");
        } else {
          addToast(t("toast.syncAllFailed"), "error");
        }
        const allStatus = await invoke<CliStatusResult[]>(
          "get_all_cli_status",
          { url }
        );
        setStatuses(allStatus);
      } catch (e: unknown) {
        const error = e instanceof Error ? e.message : String(e);
        addToast(`Sync failed: ${error}`, "error");
      } finally {
        setSyncing({});
      }
    },
    [statuses, addToast, t]
  );

  const restoreOne = useCallback(
    async (app: string, url: string, name: string) => {
      setRestoring((prev) => ({ ...prev, [app]: true }));
      try {
        await invoke("restore_cli", { app });
        addToast(t("toast.restoreSuccess", { name }), "success");
        const allStatus = await invoke<CliStatusResult[]>(
          "get_all_cli_status",
          { url }
        );
        setStatuses(allStatus);
      } catch (e: unknown) {
        const error = e instanceof Error ? e.message : String(e);
        addToast(t("toast.restoreFailed", { name, error }), "error");
      } finally {
        setRestoring((prev) => ({ ...prev, [app]: false }));
      }
    },
    [addToast, t]
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
    toasts,
    detectAll,
    syncOne,
    syncAll,
    restoreOne,
    getConfigContent,
    addToast,
    removeToast,
  };
}
