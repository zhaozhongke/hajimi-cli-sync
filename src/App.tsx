import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { toast } from "sonner";
import { Toaster } from "sonner";
import { Check, ExternalLink, Sun, Moon, RefreshCw } from "lucide-react";
import { SettingsPanel } from "./components/SettingsPanel";
import { CliCard } from "./components/CliCard";
import { ConfigViewer } from "./components/ConfigViewer";
import { ProviderPanel } from "./components/ProviderPanel";
import { useCliSync, getSyncLog } from "./hooks/useCliSync";
import type { SyncLogEntry } from "./hooks/useCliSync";
import { useModels } from "./hooks/useModels";
import { listProviders, saveProvider, switchProvider } from "./hooks/useProviders";
import { CLI_LIST } from "./types";
import type { CliInfo, CliStatusResult, ProviderRecord } from "./types";
import type { CliCategory } from "./types";

const DEFAULT_URL = "https://vip.aipro.love";
const DEFAULT_MODEL = "claude-sonnet-4-6";

function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function App() {
  const { t, i18n } = useTranslation();

  const [theme, setTheme] = useState(() => {
    const saved = localStorage.getItem("hajimi-theme");
    // Migrate old theme values
    if (!saved || saved === "emerald" || saved === "light") return "hajimi-light";
    if (saved === "dark") return "hajimi-dark";
    return saved;
  });

  // Apply theme to <html> element
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("hajimi-theme", theme);
  }, [theme]);

  const toggleTheme = useCallback(() => {
    setTheme((prev) => (prev === "hajimi-dark" ? "hajimi-light" : "hajimi-dark"));
  }, []);
  const isDark = theme === "hajimi-dark";

  // ── App version (from Tauri) ──────────────────────────────────────────────
  const [appVersion, setAppVersion] = useState("");
  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => {});
  }, []);

  // ── Provider state ──────────────────────────────────────────────────────────
  const [providers, setProviders] = useState<ProviderRecord[]>([]);
  const [isSwitching, setIsSwitching] = useState(false);
  const [providersLoading, setProvidersLoading] = useState(true);

  const reloadProviders = useCallback(async () => {
    try {
      const list = await listProviders();
      setProviders(list);
      return list;
    } catch (e) {
      console.error("Failed to load providers:", e);
      return [] as ProviderRecord[];
    }
  }, []);

  // Derived: the currently-active provider (DB truth).
  const currentProvider = providers.find((p) => p.is_current) ?? null;

  const [url, setUrl] = useState(() => localStorage.getItem("hajimi-url") || DEFAULT_URL);
  const [saveApiKey, setSaveApiKey] = useState(() => localStorage.getItem("hajimi-save-key") !== "false");
  const [apiKey, setApiKey] = useState(() =>
    localStorage.getItem("hajimi-save-key") !== "false"
      ? localStorage.getItem("hajimi-key") || ""
      : ""
  );
  const [defaultModel, setDefaultModel] = useState(() => localStorage.getItem("hajimi-model") || DEFAULT_MODEL);
  const [perCliModels, setPerCliModels] = useState<Record<string, string>>(() => {
    try {
      const saved = localStorage.getItem("hajimi-cli-models");
      return saved ? JSON.parse(saved) : {};
    } catch { return {}; }
  });

  // Sync local state when currentProvider changes (full object in deps, not just id).
  // This is the single source of truth — onSwitched callback removed from ProviderPanel.
  useEffect(() => {
    if (!currentProvider) return;
    setUrl(currentProvider.url);
    setApiKey(currentProvider.api_key);
    if (currentProvider.default_model) setDefaultModel(currentProvider.default_model);
    try {
      const pcm = JSON.parse(currentProvider.per_cli_models);
      if (pcm && typeof pcm === "object") setPerCliModels(pcm);
    } catch { /* ignore malformed JSON */ }
  }, [currentProvider]); // full object — picks up field-level changes too

  // On first load: load providers + one-time localStorage migration if DB is empty.
  useEffect(() => {
    (async () => {
      setProvidersLoading(true);
      try {
        const list = await reloadProviders();
        // Migration: if DB has no providers and localStorage has credentials,
        // create the first provider and immediately activate it.
        if (list.length === 0) {
          const lsUrl = localStorage.getItem("hajimi-url") || DEFAULT_URL;
          const lsKey = localStorage.getItem("hajimi-key") || "";
          const lsModel = localStorage.getItem("hajimi-model") || DEFAULT_MODEL;
          const lsCliModels = localStorage.getItem("hajimi-cli-models") || "{}";
          if (lsUrl && lsKey) {
            const migrated: ProviderRecord = {
              id: crypto.randomUUID(),
              name: t("provider.migratedDefault"),
              url: lsUrl,
              api_key: lsKey,
              default_model: lsModel,
              per_cli_models: lsCliModels,
              is_current: false,
              sort_index: null,
              notes: null,
              created_at: Math.floor(Date.now() / 1000),
            };
            try {
              await saveProvider(migrated);
              await switchProvider(migrated.id);
            } catch { /* non-fatal */ }
            await reloadProviders();
          }
        }
      } finally {
        setProvidersLoading(false);
      }
    })();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const [configViewer, setConfigViewer] = useState<{
    cli: CliInfo;
    status: CliStatusResult;
  } | null>(null);

  const {
    statuses,
    loading,
    syncing,
    restoring,
    detectAll,
    syncOne,
    syncAll,
    restoreOne,
    getConfigContent,
  } = useCliSync();

  const {
    models: apiModels,
    loading: modelsLoading,
    error: modelsError,
    fetchModels,
  } = useModels();

  // Persist settings (url/key/model/perCliModels still saved to localStorage as
  // fallback, and as the "live editing" state for the current session)
  useEffect(() => { localStorage.setItem("hajimi-url", url); }, [url]);
  useEffect(() => {
    localStorage.setItem("hajimi-save-key", String(saveApiKey));
    if (saveApiKey) {
      localStorage.setItem("hajimi-key", apiKey);
    } else {
      localStorage.removeItem("hajimi-key");
    }
  }, [apiKey, saveApiKey]);
  useEffect(() => { localStorage.setItem("hajimi-model", defaultModel); }, [defaultModel]);
  useEffect(() => { localStorage.setItem("hajimi-cli-models", JSON.stringify(perCliModels)); }, [perCliModels]);

  const handleUrlChange = useCallback((newUrl: string) => {
    setUrl(newUrl);
  }, []);

  // When user selects a token in account mode: upsert a Provider with the token
  // name and immediately activate it so ProviderPanel stays in sync.
  const handleAccountConfigReady = useCallback(
    async (accountUrl: string, accountApiKey: string, tokenName: string) => {
      try {
        // Check if a provider with this exact url+key already exists.
        const existing = providers.find(
          (p) => p.url === accountUrl && p.api_key === accountApiKey
        );
        let providerId: string;
        if (existing) {
          providerId = existing.id;
        } else {
          const newProvider: ProviderRecord = {
            id: crypto.randomUUID(),
            name: tokenName,
            url: accountUrl,
            api_key: accountApiKey,
            default_model: defaultModel,
            per_cli_models: "{}",
            is_current: false,
            sort_index: null,
            notes: null,
            created_at: Math.floor(Date.now() / 1000),
          };
          await saveProvider(newProvider);
          providerId = newProvider.id;
        }
        // Activate the provider — this syncs all installed CLIs.
        await switchProvider(providerId);
        await reloadProviders();
      } catch (e) {
        toast.error(String(e), { duration: 5000 });
      }
    },
    [providers, defaultModel, reloadProviders]
  );

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

  const toggleLang = useCallback(() => {
    const newLang = i18n.language === "zh" ? "en" : "zh";
    i18n.changeLanguage(newLang);
    localStorage.setItem("hajimi-lang", newLang);
  }, [i18n]);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

      if ((e.metaKey || e.ctrlKey) && e.key === "r") {
        e.preventDefault();
        detectAll(url);
      } else if (e.key === "Escape") {
        if (confirmRestoreSingle) {
          setConfirmRestoreSingle(null);
        } else if (configViewer) {
          setConfigViewer(null);
        }
      } else if (e.key === "l" && !e.metaKey && !e.ctrlKey) {
        toggleLang();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [url, configViewer, confirmRestoreSingle, detectAll, toggleLang]);

  const hasInstalled = statuses.some((s) => s.installed);

  const [activeTab, setActiveTab] = useState<CliCategory>(() =>
    (localStorage.getItem("hajimi-tab") as CliCategory) || "coding"
  );
  const [showHistory, setShowHistory] = useState(false);
  const [showManualTools, setShowManualTools] = useState(false);
  const [syncLog, setSyncLog] = useState<SyncLogEntry[]>([]);

  // Refresh log when syncing/restoring changes
  useEffect(() => {
    setSyncLog(getSyncLog());
  }, [syncing, restoring]);

  // Status dashboard counts
  const { installedCount, syncedCount } = useMemo(() => {
    const installed = statuses.filter((s) => s.installed).length;
    const synced = statuses.filter((s) => s.installed && s.is_synced).length;
    return { installedCount: installed, syncedCount: synced };
  }, [statuses]);

  // Persist active tab
  useEffect(() => { localStorage.setItem("hajimi-tab", activeTab); }, [activeTab]);

  const tabOrder: CliCategory[] = ["coding", "chat", "agent", "rp"];

  // Count installed tools per tab for badges
  const tabInstalledCounts = useMemo(() => {
    const counts: Record<CliCategory, number> = { coding: 0, chat: 0, agent: 0, rp: 0 };
    for (const cli of CLI_LIST) {
      const status = statuses.find((s) => s.app === cli.id);
      if (status?.installed) counts[cli.category]++;
    }
    return counts;
  }, [statuses]);

  // Filter CLI_LIST by active tab, installed first; split into syncable and manual-config
  const { syncableClis, manualClis } = useMemo(() => {
    const inTab = CLI_LIST.filter((c) => c.category === activeTab);
    const syncable: CliInfo[] = [];
    const manual: CliInfo[] = [];
    for (const cli of inTab) {
      if (cli.installType === "manual-config") {
        manual.push(cli);
      } else {
        syncable.push(cli);
      }
    }
    // Sort each group: installed first
    const sortInstalled = (list: CliInfo[]) => {
      const installed: CliInfo[] = [];
      const uninstalled: CliInfo[] = [];
      for (const cli of list) {
        const status = statuses.find((s) => s.app === cli.id);
        if (status?.installed) installed.push(cli);
        else uninstalled.push(cli);
      }
      return [...installed, ...uninstalled];
    };
    return { syncableClis: sortInstalled(syncable), manualClis: sortInstalled(manual) };
  }, [statuses, activeTab]);

  // Dynamic grid columns: single column when ≤ 2 tools, double when ≥ 3
  const gridCols = syncableClis.length <= 2 ? "grid-cols-1" : "grid-cols-1 md:grid-cols-2";

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
        isSwitching={isSwitching}
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
          // Deep link sync: construct URL and open it directly
          if (cli.deepLinkTemplate) {
            const config = JSON.stringify({
              id: "hakimiai",
              name: "\u54c8\u57fa\u7c73AI",
              baseUrl: url,
              apiKey: apiKey,
            });
            // btoa doesn't support Unicode, use TextEncoder for safe base64
            const bytes = new TextEncoder().encode(config);
            const base64 = btoa(String.fromCharCode(...bytes));
            const encoded = encodeURIComponent(base64);
            const deepLink = cli.deepLinkTemplate.replace("{config}", encoded);
            invoke("open_external_url", { url: deepLink });
            toast.info(t("toast.syncSuccess", { name: cli.name }));
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
        onOpenDownload={cli.downloadUrl ? () => {
          invoke("open_external_url", { url: cli.downloadUrl });
        } : undefined}
        onLaunch={cli.launchName ? () => {
          invoke("launch_app", { name: cli.launchName });
        } : undefined}
        onCommunity={cli.communityUrl ? () => {
          invoke("open_external_url", { url: cli.communityUrl });
        } : undefined}
      />
    );
  };

  return (
    <div className="min-h-screen mesh-bg p-4 md:p-6">
      <div className="max-w-5xl mx-auto md:flex md:gap-6">
        {/* Left column: Header + Settings (sticky on wide screens, scrollable) */}
        <div className="md:w-80 md:shrink-0 md:sticky md:top-6 md:self-start md:max-h-[calc(100vh-3rem)] md:overflow-y-auto space-y-4">
          {/* Branded Header — two rows for clarity */}
          <div className="space-y-2">
            <div className="flex items-center gap-3">
              {/* App logo — exact match to app icon: indigo→violet→cyan gradient + lightning */}
              <svg
                width="36" height="36" viewBox="0 0 512 512"
                xmlns="http://www.w3.org/2000/svg"
                className="shrink-0 shadow-md rounded-xl"
                style={{ filter: "drop-shadow(0 2px 6px #6366f140)" }}
              >
                <defs>
                  <linearGradient id="app-logo-grad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%"   stopColor="#6366f1" />
                    <stop offset="50%"  stopColor="#8b5cf6" />
                    <stop offset="100%" stopColor="#06b6d4" />
                  </linearGradient>
                </defs>
                <rect width="512" height="512" rx="108" ry="108" fill="url(#app-logo-grad)" />
                <g transform="translate(256,256) scale(13) translate(-12,-12)" fill="white" stroke="none">
                  <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
                </g>
              </svg>
              <div className="min-w-0 flex-1">
                <h1 className="text-base font-bold leading-tight tracking-tight truncate">
                  {t("app.title")}
                  <span className="text-[10px] font-normal opacity-30 ml-1">{appVersion && `v${appVersion}`}</span>
                </h1>
                <p className="text-[11px] opacity-50 leading-tight truncate">{t("app.subtitle")}</p>
              </div>
            </div>
            {/* Controls row */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-1.5">
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
                  className="btn btn-ghost btn-xs gap-1 opacity-50 hover:opacity-100 transition-opacity"
                  onClick={() => invoke("open_external_url", { url: "https://docs.aipro.love" })}
                >
                  {t("app.docs")}
                  <ExternalLink className="w-2.5 h-2.5" />
                </button>
              </div>
              <div className="flex items-center gap-0.5">
                <button
                  className="btn btn-ghost btn-xs btn-square opacity-50 hover:opacity-100 transition-opacity"
                  onClick={toggleTheme}
                  title={isDark ? t("app.lightMode") : t("app.darkMode")}
                >
                  {isDark ? <Sun className="w-3.5 h-3.5" /> : <Moon className="w-3.5 h-3.5" />}
                </button>
                <button
                  className="btn btn-ghost btn-xs opacity-50 hover:opacity-100 transition-opacity"
                  onClick={toggleLang}
                >
                  {i18n.language === "zh" ? "EN" : "\u4e2d\u6587"}
                </button>
              </div>
            </div>
          </div>

          {/* Provider Panel */}
          <div className="card glass-card shadow-lg">
            <div className="card-body p-4">
              {providersLoading ? (
                <div className="flex items-center justify-center py-4 opacity-40">
                  <span className="loading loading-spinner loading-xs mr-2" />
                  <span className="text-xs">{t("provider.loading")}</span>
                </div>
              ) : (
                <ProviderPanel
                  providers={providers}
                  onProvidersChange={reloadProviders}
                  onSwitched={() => {
                    // State is driven exclusively by useEffect[currentProvider].
                    // detectAll is triggered by the url change in that effect.
                    // No manual state updates here — single source of truth.
                  }}
                  isSwitching={isSwitching}
                  setIsSwitching={setIsSwitching}
                />
              )}
            </div>
          </div>

          {/* Settings Card */}
          <div className="card glass-card shadow-lg">
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
                saveApiKey={saveApiKey}
                onSaveApiKeyChange={setSaveApiKey}
                onAccountConfigReady={handleAccountConfigReady}
              />
            </div>
          </div>
        </div>

        {/* Right column: Tabs + Tool cards + History */}
        <div className="md:flex-1 md:min-w-0 space-y-4 mt-4 md:mt-0">
          {/* Sync status bar + Sync All + Refresh */}
          <div className="flex items-center gap-2">
            <div className="tabs tabs-boxed glass-card p-1.5 shadow-sm flex-1">
              {tabOrder.map((tab) => {
                const count = tabInstalledCounts[tab];
                const isActive = activeTab === tab;
                return (
                  <button
                    key={tab}
                    className={`tab tab-sm tab-glow flex-1 gap-1.5 transition-all ${isActive ? "tab-active !bg-primary !text-primary-content font-semibold shadow-sm" : "hover:bg-base-200/50"}`}
                    onClick={() => setActiveTab(tab)}
                  >
                    {t(`category.${tab}`)}
                    {count > 0 && (
                      <span className={`badge badge-xs ${isActive ? "bg-primary-content/20 text-primary-content border-0" : "badge-ghost"}`}>
                        {count}
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
            {/* Sync All — primary CTA when tools are installed but not all synced */}
            {hasInstalled && (
              <button
                className={`btn btn-sm shrink-0 gap-1.5 ${
                  syncedCount === installedCount
                    ? "btn-success btn-outline"
                    : "btn-primary"
                }`}
                onClick={() => {
                  if (!apiKey) { toast.error(t("toast.apiKeyRequired")); return; }
                  syncAll(url, apiKey, defaultModel, perCliModels);
                }}
                disabled={loading || Object.values(syncing).some(Boolean) || isSwitching || !url.trim() || !apiKey.trim()}
                title={t("settings.syncAll")}
              >
                {Object.values(syncing).some(Boolean)
                  ? <span className="loading loading-spinner loading-xs" />
                  : syncedCount === installedCount
                  ? <Check className="w-3.5 h-3.5" />
                  : null}
                {t("settings.syncAll")}
              </button>
            )}
            <button
              className="btn btn-ghost btn-sm btn-square opacity-50 hover:opacity-100 transition-opacity shrink-0"
              onClick={() => detectAll(url)}
              disabled={loading || isSwitching}
              title={t("cli.refresh")}
            >
              {loading
                ? <span className="loading loading-spinner loading-xs" />
                : <RefreshCw className="w-4 h-4" />
              }
            </button>
          </div>

          {/* Switching banner — shown while a provider switch is in progress */}
          {isSwitching && (
            <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-primary/10 border border-primary/20 text-primary text-xs">
              <span className="loading loading-spinner loading-xs shrink-0" />
              {t("provider.switching")}
            </div>
          )}

          {/* Tool cards for active tab (syncable tools) */}
          <div className={`grid ${gridCols} gap-3`}>
            {syncableClis.map(renderCliCard)}
          </div>

          {/* Empty state: no tools installed in this tab */}
          {!loading && syncableClis.length > 0 &&
            syncableClis.every((c) => !statuses.find((s) => s.app === c.id)?.installed) &&
            manualClis.every((c) => !statuses.find((s) => s.app === c.id)?.installed) && (
            <div className="card glass-card shadow-sm">
              <div className="card-body items-center text-center py-8 gap-2">
                <p className="text-sm font-medium opacity-60">{t("empty.title")}</p>
                <p className="text-xs opacity-40 max-w-xs">{t("empty.description")}</p>
              </div>
            </div>
          )}

          {/* Manual-config tools — collapsible, hidden by default */}
          {manualClis.length > 0 && (
            <div className="space-y-2">
              <button
                className="btn btn-ghost btn-sm w-full justify-between opacity-50 hover:opacity-100 transition-opacity"
                onClick={() => setShowManualTools(!showManualTools)}
              >
                <span className="text-xs">
                  {t("section.manualConfig")}
                  <span className="badge badge-ghost badge-xs ml-1.5">{t("section.manualConfigCount", { count: manualClis.length })}</span>
                </span>
                <span className={`text-xs transition-transform duration-200 ${showManualTools ? "rotate-180" : ""}`}>
                  {"▼"}
                </span>
              </button>
              {showManualTools && (
                <div className="grid grid-cols-1 gap-3">
                  {manualClis.map(renderCliCard)}
                </div>
              )}
            </div>
          )}

          {/* Sync History */}
          {syncLog.length > 0 && (
            <div className="space-y-2">
              <button
                className="btn btn-ghost btn-sm w-full justify-between opacity-50 hover:opacity-100 transition-opacity"
                onClick={() => setShowHistory(!showHistory)}
              >
                <span className="text-xs">{t("history.title")}</span>
                <span className={`text-xs transition-transform duration-200 ${showHistory ? "rotate-180" : ""}`}>
                  {"▼"}
                </span>
              </button>
              {showHistory && (
                <div className="card glass-card shadow-sm">
                  <div className="card-body p-3 gap-1 max-h-48 overflow-auto">
                    {syncLog.slice(0, 20).map((entry) => (
                      <div key={entry.id} className="flex items-center gap-2 text-xs">
                        <span className={entry.success ? "text-success" : "text-error"}>
                          {entry.success ? "✓" : "✗"}
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

      {/* Toast notifications — success 2.5s, errors 5s */}
      <Toaster position="bottom-right" richColors duration={2500} />
    </div>
  );
}

export default App;
