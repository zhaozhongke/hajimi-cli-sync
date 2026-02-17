import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { SettingsPanel } from "./components/SettingsPanel";
import { CliCard } from "./components/CliCard";
import { ConfigViewer } from "./components/ConfigViewer";
import { useCliSync } from "./hooks/useCliSync";
import { useModels } from "./hooks/useModels";
import { CLI_LIST } from "./types";
import type { CliInfo, CliStatusResult } from "./types";

const DEFAULT_URL = "https://free.aipro.love";
const DEFAULT_MODEL = "claude-sonnet-4-5";

function App() {
  const { t, i18n } = useTranslation();

  const [url, setUrl] = useState(() => localStorage.getItem("hajimi-url") || DEFAULT_URL);
  const [apiKey, setApiKey] = useState(() => localStorage.getItem("hajimi-key") || "");
  const [defaultModel, setDefaultModel] = useState(() => localStorage.getItem("hajimi-model") || DEFAULT_MODEL);
  const [perCliModels, setPerCliModels] = useState<Record<string, string>>(() => {
    try {
      const saved = localStorage.getItem("hajimi-cli-models");
      return saved ? JSON.parse(saved) : {};
    } catch { return {}; }
  });

  const [configViewer, setConfigViewer] = useState<{
    cli: CliInfo;
    status: CliStatusResult;
  } | null>(null);

  const {
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
  } = useCliSync();

  const {
    models: apiModels,
    loading: modelsLoading,
    error: modelsError,
    fetchModels,
  } = useModels();

  // Persist settings
  useEffect(() => { localStorage.setItem("hajimi-url", url); }, [url]);
  useEffect(() => { localStorage.setItem("hajimi-key", apiKey); }, [apiKey]);
  useEffect(() => { localStorage.setItem("hajimi-model", defaultModel); }, [defaultModel]);
  useEffect(() => { localStorage.setItem("hajimi-cli-models", JSON.stringify(perCliModels)); }, [perCliModels]);

  // Detect CLIs on mount and when URL changes
  useEffect(() => {
    detectAll(url);
  }, []);

  const handleUrlChange = useCallback((newUrl: string) => {
    setUrl(newUrl);
  }, []);

  // Re-detect when URL changes (debounced)
  useEffect(() => {
    const timer = setTimeout(() => {
      detectAll(url);
    }, 500);
    return () => clearTimeout(timer);
  }, [url, detectAll]);

  // Fetch models when URL and API key are both available (debounced)
  useEffect(() => {
    if (!url.trim() || !apiKey.trim()) return;
    const timer = setTimeout(() => {
      fetchModels(url, apiKey);
    }, 800);
    return () => clearTimeout(timer);
  }, [url, apiKey, fetchModels]);

  const getModelForCli = (appId: string) => perCliModels[appId] || defaultModel;

  const handleSyncAll = () => {
    if (!apiKey) {
      addToast(t("toast.apiKeyRequired"), "error");
      return;
    }
    const hasInstalled = statuses.some((s) => s.installed);
    if (!hasInstalled) {
      addToast(t("toast.noInstalledCli"), "error");
      return;
    }
    syncAll(url, apiKey, defaultModel);
  };

  const [confirmRestore, setConfirmRestore] = useState(false);

  const handleRestoreAll = async () => {
    const installed = statuses.filter((s) => s.installed && s.has_backup);
    if (installed.length === 0) {
      addToast(t("toast.noBackups"), "error");
      return;
    }
    setConfirmRestore(true);
  };

  const doRestoreAll = async () => {
    setConfirmRestore(false);
    const installed = statuses.filter((s) => s.installed && s.has_backup);
    for (const s of installed) {
      const cli = CLI_LIST.find((c) => c.id === s.app);
      if (cli) {
        await restoreOne(s.app, url, cli.name);
      }
    }
  };

  const toggleLang = () => {
    const newLang = i18n.language === "zh" ? "en" : "zh";
    i18n.changeLanguage(newLang);
    localStorage.setItem("hajimi-lang", newLang);
  };

  const hasInstalled = statuses.some((s) => s.installed);
  const isSyncingAny = Object.values(syncing).some(Boolean);

  return (
    <div className="min-h-screen bg-base-200 p-4">
      <div className="max-w-xl mx-auto space-y-4">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold">{t("app.title")}</h1>
            <p className="text-xs opacity-60">{t("app.subtitle")}</p>
          </div>
          <button
            className="btn btn-ghost btn-sm"
            onClick={toggleLang}
          >
            {i18n.language === "zh" ? "EN" : "中文"}
          </button>
        </div>

        {/* Settings */}
        <div className="card bg-base-100 shadow-sm">
          <div className="card-body p-4">
            <SettingsPanel
              url={url}
              apiKey={apiKey}
              defaultModel={defaultModel}
              onUrlChange={handleUrlChange}
              onApiKeyChange={setApiKey}
              onModelChange={setDefaultModel}
              onSyncAll={handleSyncAll}
              onRestoreAll={handleRestoreAll}
              syncing={isSyncingAny}
              hasInstalled={hasInstalled}
              apiModels={apiModels}
              modelsLoading={modelsLoading}
              modelsError={modelsError}
            />
          </div>
        </div>

        {/* CLI Cards */}
        <div className="space-y-2">
          {CLI_LIST.map((cli) => {
            const status = statuses.find((s) => s.app === cli.id);
            return (
              <CliCard
                key={cli.id}
                cli={cli}
                status={status}
                loading={loading}
                syncing={syncing[cli.id] || false}
                restoring={restoring[cli.id] || false}
                model={getModelForCli(cli.id)}
                onModelChange={(m) =>
                  setPerCliModels((prev) => ({ ...prev, [cli.id]: m }))
                }
                apiModels={apiModels}
                modelsLoading={modelsLoading}
                onSync={() => {
                  if (!apiKey) {
                    addToast(t("toast.apiKeyRequired"), "error");
                    return;
                  }
                  syncOne(cli.id, url, apiKey, getModelForCli(cli.id), cli.name);
                }}
                onRestore={() => restoreOne(cli.id, url, cli.name)}
                onViewConfig={() => {
                  if (status) {
                    setConfigViewer({ cli, status });
                  }
                }}
              />
            );
          })}
        </div>
      </div>

      {/* Config Viewer Modal */}
      {configViewer && (
        <ConfigViewer
          name={configViewer.cli.name}
          files={configViewer.status.files}
          getContent={(fileName) =>
            getConfigContent(configViewer.cli.id, fileName)
          }
          onClose={() => setConfigViewer(null)}
        />
      )}

      {/* Restore All Confirmation */}
      {confirmRestore && (
        <div className="modal modal-open">
          <div className="modal-box max-w-sm">
            <h3 className="font-bold text-lg">{t("confirm.restoreTitle")}</h3>
            <p className="py-4 text-sm">{t("confirm.restoreMessage")}</p>
            <div className="modal-action">
              <button
                className="btn btn-sm"
                onClick={() => setConfirmRestore(false)}
              >
                {t("confirm.cancel")}
              </button>
              <button
                className="btn btn-warning btn-sm"
                onClick={doRestoreAll}
              >
                {t("confirm.restore")}
              </button>
            </div>
          </div>
          <div className="modal-backdrop" onClick={() => setConfirmRestore(false)} />
        </div>
      )}

      {/* Toast notifications */}
      <div className="toast toast-end toast-bottom z-50">
        {toasts.map((toast) => (
          <div
            key={toast.id}
            className={`alert ${
              toast.type === "success" ? "alert-success" : "alert-error"
            } shadow-lg cursor-pointer text-sm py-2 px-4`}
            onClick={() => removeToast(toast.id)}
          >
            <span>{toast.msg}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export default App;
