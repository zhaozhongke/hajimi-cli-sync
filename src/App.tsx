import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Toaster } from "sonner";
import { Zap, Check } from "lucide-react";
import { SettingsPanel } from "./components/SettingsPanel";
import { CliCard } from "./components/CliCard";
import { ConfigViewer } from "./components/ConfigViewer";
import { useCliSync, getSyncLog } from "./hooks/useCliSync";
import type { SyncLogEntry } from "./hooks/useCliSync";
import { useModels } from "./hooks/useModels";
import { CLI_LIST } from "./types";
import type { CliInfo, CliStatusResult, CliCategory } from "./types";

const DEFAULT_URL = "https://free.aipro.love";
const DEFAULT_MODEL = "claude-sonnet-4-6";
const APP_VERSION = "1.0.0";

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
    installing,
    detectAll,
    syncOne,
    restoreOne,
    installOne,
    getConfigContent,
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

  // Detect CLIs on mount
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

  const [confirmRestoreSingle, setConfirmRestoreSingle] = useState<{
    app: string;
    name: string;
  } | null>(null);

  const toggleLang = () => {
    const newLang = i18n.language === "zh" ? "en" : "zh";
    i18n.changeLanguage(newLang);
    localStorage.setItem("hajimi-lang", newLang);
  };

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

      if ((e.metaKey || e.ctrlKey) && e.key === "r") {
        e.preventDefault();
        detectAll(url);
      } else if (e.key === "Escape" && configViewer) {
        setConfigViewer(null);
      } else if (e.key === "l" && !e.metaKey && !e.ctrlKey) {
        toggleLang();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [url, configViewer, detectAll, toggleLang]);

  const hasInstalled = statuses.some((s) => s.installed);

  const [showMore, setShowMore] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const [syncLog, setSyncLog] = useState<SyncLogEntry[]>([]);

  // Refresh log when syncing/restoring/installing changes
  useEffect(() => {
    setSyncLog(getSyncLog());
  }, [syncing, restoring, installing]);

  // Status dashboard counts
  const { installedCount, syncedCount } = useMemo(() => {
    const installed = statuses.filter((s) => s.installed).length;
    const synced = statuses.filter((s) => s.installed && s.is_synced).length;
    return { installedCount: installed, syncedCount: synced };
  }, [statuses]);

  // Split CLIs into installed vs uninstalled, group uninstalled by category
  const { installedClis, uninstalledByCategory, uninstalledCount } = useMemo(() => {
    const installed: CliInfo[] = [];
    const uninstalled: CliInfo[] = [];
    for (const cli of CLI_LIST) {
      const status = statuses.find((s) => s.app === cli.id);
      if (status?.installed) {
        installed.push(cli);
      } else {
        uninstalled.push(cli);
      }
    }
    const byCategory: Record<CliCategory, CliInfo[]> = { cli: [], desktop: [], extension: [] };
    for (const cli of uninstalled) {
      byCategory[cli.category].push(cli);
    }
    return { installedClis: installed, uninstalledByCategory: byCategory, uninstalledCount: uninstalled.length };
  }, [statuses]);

  // Show Quick Start when no CLIs are installed
  const showQuickStart = !loading && !hasInstalled;
  const claudeCli = CLI_LIST.find((c) => c.id === "claude")!;

  const renderCliCard = (cli: CliInfo) => {
    const status = statuses.find((s) => s.app === cli.id);
    return (
      <CliCard
        key={cli.id}
        cli={cli}
        status={status}
        loading={loading}
        syncing={syncing[cli.id] || false}
        restoring={restoring[cli.id] || false}
        installing={installing[cli.id] || false}
        model={getModelForCli(cli.id)}
        onModelChange={(m) =>
          setPerCliModels((prev) => ({ ...prev, [cli.id]: m }))
        }
        apiModels={apiModels}
        modelsLoading={modelsLoading}
        onSync={() => {
          if (!apiKey) {
            toast.error(t("toast.apiKeyRequired"));
            return;
          }
          syncOne(cli.id, url, apiKey, getModelForCli(cli.id), cli.name);
        }}
        onRestore={() => {
          if (status) {
            setConfirmRestoreSingle({ app: cli.id, name: cli.name });
          }
        }}
        onViewConfig={() => {
          if (status) {
            setConfigViewer({ cli, status });
          }
        }}
        onInstall={() => {
          installOne(cli.id, url, cli.name);
        }}
        onDownload={() => {
          if (cli.downloadUrl) {
            invoke("open_external_url", { url: cli.downloadUrl });
          }
        }}
      />
    );
  };

  const categoryOrder: CliCategory[] = ["cli", "desktop", "extension"];

  return (
    <div className="min-h-screen bg-base-200 p-4">
      <div className="max-w-2xl mx-auto space-y-4">
        {/* Branded Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-primary to-secondary flex items-center justify-center text-primary-content">
              <Zap className="w-5 h-5" />
            </div>
            <div>
              <h1 className="text-lg font-bold leading-tight">
                {t("app.title")}
                <span className="text-[10px] font-normal opacity-30 ml-1.5">v{APP_VERSION}</span>
              </h1>
              <p className="text-[11px] opacity-50 leading-tight">{t("app.subtitle")}</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            {/* Status pill */}
            {!loading && hasInstalled && (
              <div className={`badge badge-sm gap-1 ${syncedCount > 0 ? "badge-success badge-outline" : "badge-ghost"}`}>
                {syncedCount > 0 ? (
                  <>
                    <Check className="w-2.5 h-2.5" />
                    {`${syncedCount}/${installedCount}`}
                  </>
                ) : (
                  t("app.statusNone")
                )}
              </div>
            )}
            <button
              className="btn btn-ghost btn-xs"
              onClick={toggleLang}
            >
              {i18n.language === "zh" ? "EN" : "\u4e2d\u6587"}
            </button>
          </div>
        </div>

        {/* Settings Card */}
        <div className="card bg-base-100 shadow-sm">
          <div className="card-body p-4">
            <SettingsPanel
              url={url}
              apiKey={apiKey}
              defaultModel={defaultModel}
              onUrlChange={handleUrlChange}
              onApiKeyChange={setApiKey}
              onModelChange={setDefaultModel}
              apiModels={apiModels}
              modelsLoading={modelsLoading}
              modelsError={modelsError}
              perCliModels={perCliModels}
              onPerCliModelsChange={setPerCliModels}
            />
          </div>
        </div>

        {/* Quick Start Banner */}
        {showQuickStart && (
          <div className="card bg-primary/10 border border-primary/20 shadow-sm">
            <div className="card-body p-4 gap-2">
              <h2 className="text-sm font-semibold">{t("install.quickStart")}</h2>
              <p className="text-xs opacity-70">{t("install.quickStartDesc")}</p>
              <button
                className="btn btn-primary btn-sm w-fit"
                onClick={() => installOne(claudeCli.id, url, claudeCli.name)}
                disabled={installing["claude"] || false}
              >
                {installing["claude"] ? (
                  <span className="loading loading-spinner loading-xs" />
                ) : null}
                {installing["claude"] ? t("install.installing") : t("install.install")}
              </button>
            </div>
          </div>
        )}

        {/* Installed CLIs */}
        {installedClis.length > 0 && (
          <div className="space-y-2">
            <div className="flex items-center gap-2 px-1">
              <span className="text-xs font-medium opacity-50">{t("section.installed")}</span>
              <div className="flex-1 h-px bg-base-300" />
              <span className="text-[10px] opacity-30">{installedClis.length}</span>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
              {installedClis.map(renderCliCard)}
            </div>
          </div>
        )}

        {/* Uninstalled CLIs -- collapsible by category */}
        {!loading && uninstalledCount > 0 && (
          <div className="space-y-2">
            <button
              className="btn btn-ghost btn-sm w-full justify-between opacity-60 hover:opacity-100"
              onClick={() => setShowMore(!showMore)}
            >
              <span className="text-xs">
                {showMore
                  ? t("section.showLess")
                  : t("section.showMore", { count: uninstalledCount })}
              </span>
              <span className={`text-xs transition-transform ${showMore ? "rotate-180" : ""}`}>
                {"\u25bc"}
              </span>
            </button>

            {showMore && (
              <div className="space-y-3">
                {categoryOrder.map((cat) => {
                  const clis = uninstalledByCategory[cat];
                  if (clis.length === 0) return null;
                  return (
                    <div key={cat}>
                      <div className="flex items-center gap-2 px-1 mb-1.5">
                        <span className="text-xs font-medium opacity-40">
                          {t(`category.${cat}`)}
                        </span>
                        <div className="flex-1 h-px bg-base-300" />
                      </div>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                        {clis.map(renderCliCard)}
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        )}

        {/* Sync History */}
        {syncLog.length > 0 && (
          <div className="space-y-1">
            <button
              className="btn btn-ghost btn-sm w-full justify-between opacity-60 hover:opacity-100"
              onClick={() => setShowHistory(!showHistory)}
            >
              <span className="text-xs">{t("history.title")}</span>
              <span className={`text-xs transition-transform ${showHistory ? "rotate-180" : ""}`}>
                {"\u25bc"}
              </span>
            </button>
            {showHistory && (
              <div className="card bg-base-100 shadow-sm">
                <div className="card-body p-3 gap-1 max-h-48 overflow-auto">
                  {syncLog.slice(0, 20).map((entry) => (
                    <div key={entry.id} className="flex items-center gap-2 text-xs">
                      <span className={entry.success ? "text-success" : "text-error"}>
                        {entry.success ? "\u2713" : "\u2717"}
                      </span>
                      <span className="opacity-50 font-mono shrink-0">
                        {new Date(entry.time).toLocaleTimeString()}
                      </span>
                      <span className="badge badge-ghost badge-xs">
                        {t(`history.${entry.action}`)}
                      </span>
                      <span className="truncate">{entry.app}</span>
                      {entry.detail && (
                        <span className="opacity-40 truncate">{entry.detail}</span>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
      {configViewer && (
        <ConfigViewer
          name={configViewer.cli.name}
          files={configViewer.status.files}
          getContent={(fileName) =>
            getConfigContent(configViewer.cli.id, fileName)
          }
          onClose={() => setConfigViewer(null)}
          cliId={configViewer.cli.id}
        />
      )}

      {/* Restore Single Confirmation */}
      {confirmRestoreSingle && (
        <div className="modal modal-open">
          <div className="modal-box max-w-sm">
            <h3 className="font-bold text-lg">{t("confirm.restoreTitle")}</h3>
            <p className="py-4 text-sm">
              {t("confirm.restoreSingleMessage", { name: confirmRestoreSingle.name })}
            </p>
            <div className="modal-action">
              <button
                className="btn btn-sm"
                onClick={() => setConfirmRestoreSingle(null)}
              >
                {t("confirm.cancel")}
              </button>
              <button
                className="btn btn-warning btn-sm"
                onClick={() => {
                  const { app, name } = confirmRestoreSingle;
                  setConfirmRestoreSingle(null);
                  restoreOne(app, url, name);
                }}
              >
                {t("confirm.restore")}
              </button>
            </div>
          </div>
          <div className="modal-backdrop" onClick={() => setConfirmRestoreSingle(null)} />
        </div>
      )}

      {/* Toast notifications */}
      <Toaster position="bottom-right" richColors duration={2500} />
    </div>
  );
}

export default App;
