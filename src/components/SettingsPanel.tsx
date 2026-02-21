import { useState, useMemo, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { save, open } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import { Eye, EyeOff, Check, X, RefreshCw, Download, Upload, KeyRound, UserCircle, ShoppingCart } from "lucide-react";
import { toast } from "sonner";
import { ModelSelector } from "./ModelSelector";
import { AccountLogin } from "./AccountLogin";
import type { AuthMode } from "../types";

interface SettingsPanelProps {
  url: string;
  apiKey: string;
  defaultModel: string;
  onUrlChange: (url: string) => void;
  onApiKeyChange: (key: string) => void;
  onModelChange: (model: string) => void;
  apiModels: string[];
  modelsLoading: boolean;
  modelsError: string | null;
  perCliModels: Record<string, string>;
  onPerCliModelsChange: (models: Record<string, string>) => void;
  saveApiKey: boolean;
  onSaveApiKeyChange: (save: boolean) => void;
  /** Called when user selects a token in account mode — includes token name for Provider naming */
  onAccountConfigReady: (url: string, apiKey: string, tokenName: string) => void;
}

export function SettingsPanel({
  url,
  apiKey,
  defaultModel,
  onUrlChange,
  onApiKeyChange,
  onModelChange,
  apiModels,
  modelsLoading,
  modelsError,
  perCliModels,
  onPerCliModelsChange,
  saveApiKey,
  onSaveApiKeyChange,
  onAccountConfigReady,
}: SettingsPanelProps) {
  const { t } = useTranslation();
  const [authMode, setAuthMode] = useState<AuthMode>(
    () => (localStorage.getItem("hajimi-auth-mode") as AuthMode) || "manual"
  );
  const [showKey, setShowKey] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<"success" | "error" | null>(null);

  const handleAuthModeChange = (mode: AuthMode) => {
    setAuthMode(mode);
    localStorage.setItem("hajimi-auth-mode", mode);
  };

  const urlError = useMemo(() => {
    const trimmed = url.trim();
    if (!trimmed) return t("settings.urlRequired");
    if (!trimmed.startsWith("http://") && !trimmed.startsWith("https://"))
      return t("settings.urlInvalid");
    return null;
  }, [url, t]);

  // Warn when user enters an http:// (non-TLS) URL — credentials sent in cleartext
  const prevUrlRef = useRef<string>("");
  useEffect(() => {
    const trimmed = url.trim();
    if (
      trimmed.startsWith("http://") &&
      !prevUrlRef.current.startsWith("http://")
    ) {
      toast.warning(t("settings.httpWarning"));
    }
    prevUrlRef.current = trimmed;
  }, [url, t]);

  // Determine which step user is on (manual mode only)
  const currentStep = useMemo(() => {
    if (!url.trim() || urlError) return 1;
    if (!apiKey.trim()) return 2;
    return 3;
  }, [url, apiKey, urlError]);

  const handleTestConnection = async () => {
    if (!url.trim() || !apiKey.trim()) return;
    setTesting(true);
    setTestResult(null);
    try {
      await invoke("test_connection", { url, apiKey });
      setTestResult("success");
    } catch {
      setTestResult("error");
    } finally {
      setTesting(false);
      setTimeout(() => setTestResult(null), 3000);
    }
  };

  const handleExportSettings = async () => {
    const data = {
      version: 1,
      url,
      apiKey,
      defaultModel,
      perCliModels,
    };
    try {
      const filePath = await save({
        defaultPath: "hajimi-settings.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (filePath) {
        await writeTextFile(filePath, JSON.stringify(data, null, 2));
        toast.success(t("settings.exportSuccess"));
      }
    } catch (e) {
      toast.error(t("settings.exportFailed") + ": " + e);
    }
  };

  const handleImportSettings = async () => {
    try {
      const filePath = await open({
        filters: [{ name: "JSON", extensions: ["json"] }],
        multiple: false,
      });
      if (!filePath) return;
      const raw = await readTextFile(filePath as string);
      const data = JSON.parse(raw);
      if (data.url) onUrlChange(data.url);
      if (data.apiKey) onApiKeyChange(data.apiKey);
      if (data.defaultModel) onModelChange(data.defaultModel);
      if (data.perCliModels) onPerCliModelsChange(data.perCliModels);
      toast.success(t("settings.importSuccess"));
    } catch (e) {
      toast.error(t("settings.importFailed") + ": " + e);
    }
  };

  const handleAccountConfigReady = (accountUrl: string, accountApiKey: string, tokenName: string) => {
    onUrlChange(accountUrl);
    onApiKeyChange(accountApiKey);
    onAccountConfigReady(accountUrl, accountApiKey, tokenName);
  };

  const stepIndicator = (step: number, label: string) => {
    const isActive = currentStep === step;
    const isDone = currentStep > step;
    return (
      <div className="flex items-center gap-1.5">
        <div
          className={`w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold transition-all ${
            isDone
              ? "bg-success text-success-content"
              : isActive
              ? "bg-primary text-primary-content"
              : "bg-base-300 text-base-content/40"
          }`}
        >
          {isDone ? <Check className="w-3 h-3" /> : step}
        </div>
        <span
          className={`text-xs font-medium ${
            isActive ? "text-base-content" : isDone ? "text-success" : "text-base-content/40"
          }`}
        >
          {label}
        </span>
      </div>
    );
  };

  return (
    <div className="space-y-3">
      {/* Auth mode tabs */}
      <div className="flex rounded-lg bg-base-200 p-0.5">
        <button
          className={`flex-1 btn btn-xs gap-1.5 ${
            authMode === "manual"
              ? "btn-primary"
              : "btn-ghost"
          }`}
          onClick={() => handleAuthModeChange("manual")}
        >
          <KeyRound className="w-3 h-3" />
          {t("account.modeManual")}
        </button>
        <button
          className={`flex-1 btn btn-xs gap-1.5 ${
            authMode === "account"
              ? "btn-primary"
              : "btn-ghost"
          }`}
          onClick={() => handleAuthModeChange("account")}
        >
          <UserCircle className="w-3 h-3" />
          {t("account.modeAccount")}
        </button>
      </div>

      {/* Manual mode */}
      {authMode === "manual" && (
        <>
          {/* Step indicators */}
          <div className="flex items-center gap-4 pb-1">
            {stepIndicator(1, t("steps.step1"))}
            <div className="flex-1 h-px bg-base-300" />
            {stepIndicator(2, t("steps.step2"))}
            <div className="flex-1 h-px bg-base-300" />
            {stepIndicator(3, t("steps.step3"))}
          </div>

          {/* Step 1: URL */}
          <div className="form-control">
            <input
              type="text"
              className={`input input-bordered input-sm w-full ${urlError ? "input-error" : currentStep === 1 ? "input-primary" : ""}`}
              value={url}
              onChange={(e) => onUrlChange(e.target.value)}
              placeholder={t("steps.step1Hint")}
            />
            {urlError && (
              <span className="label-text-alt text-error text-xs mt-1">{urlError}</span>
            )}
          </div>

          {/* Step 2: API Key */}
          <div className="form-control">
            <div className="join w-full">
              <input
                type={showKey ? "text" : "password"}
                className={`input input-bordered input-sm join-item w-full ${currentStep === 2 ? "input-primary" : ""}`}
                value={apiKey}
                onChange={(e) => onApiKeyChange(e.target.value)}
                placeholder={t("steps.step2Hint")}
              />
              <button
                className="btn btn-sm btn-ghost join-item"
                onClick={() => setShowKey(!showKey)}
                aria-label={showKey ? "Hide API key" : "Show API key"}
              >
                {showKey ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
              </button>
              <button
                className={`btn btn-sm join-item ${
                  testResult === "success"
                    ? "btn-success"
                    : testResult === "error"
                    ? "btn-error"
                    : "btn-ghost"
                }`}
                onClick={handleTestConnection}
                disabled={testing || !!urlError || !apiKey.trim()}
                title={t("connection.test")}
              >
                {testing ? (
                  <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                ) : testResult === "success" ? (
                  <Check className="w-3.5 h-3.5" />
                ) : testResult === "error" ? (
                  <X className="w-3.5 h-3.5" />
                ) : (
                  t("connection.test")
                )}
              </button>
            </div>
          </div>

          {/* Connection test failure hint */}
          {testResult === "error" && (
            <div className="text-xs text-error opacity-70 px-0.5">
              {t("connection.failedHint")}
            </div>
          )}

          {/* Remember key toggle */}
          <div className="flex items-center justify-between px-0.5">
            <label className="flex items-center gap-1.5 cursor-pointer select-none" title={t("settings.saveApiKeyHint")}>
              <input
                type="checkbox"
                className="toggle toggle-xs toggle-primary"
                checked={saveApiKey}
                onChange={(e) => onSaveApiKeyChange(e.target.checked)}
              />
              <span className="text-xs opacity-60">{t("settings.saveApiKey")}</span>
            </label>
            {!saveApiKey && (
              <span className="text-[10px] opacity-40 italic">{t("settings.saveApiKeyHint")}</span>
            )}
          </div>

          {/* Step 3: Default Model */}
          <div className="form-control">
            <ModelSelector
              value={defaultModel}
              onChange={onModelChange}
              apiModels={apiModels}
              modelsLoading={modelsLoading}
              modelsError={modelsError}
              size="sm"
            />
          </div>

          {/* Import / Export */}
          <div className="flex gap-2">
            <button
              className="btn btn-ghost btn-xs flex-1 gap-1 opacity-60"
              onClick={handleExportSettings}
            >
              <Download className="w-3 h-3" />
              {t("settings.export")}
            </button>
            <button
              className="btn btn-ghost btn-xs flex-1 gap-1 opacity-60"
              onClick={handleImportSettings}
            >
              <Upload className="w-3 h-3" />
              {t("settings.import")}
            </button>
          </div>

          {/* Purchase CTA */}
          <button
            className="w-full flex items-center gap-2.5 px-3 py-2.5 rounded-xl bg-gradient-to-r from-orange-500/10 to-amber-500/10 border border-orange-400/20 hover:border-orange-400/40 hover:from-orange-500/15 hover:to-amber-500/15 transition-all text-left group"
            onClick={() => invoke("open_external_url", { url: "https://m.tb.cn/h.7EJM4va?tk=hb16UmTYhKB" })}
          >
            <div className="w-7 h-7 rounded-lg bg-orange-500/15 flex items-center justify-center shrink-0 group-hover:bg-orange-500/25 transition-colors">
              <ShoppingCart className="w-3.5 h-3.5 text-orange-500" />
            </div>
            <div className="min-w-0 flex-1">
              <div className="text-xs font-semibold text-orange-500/90 leading-tight">{t("purchase.title")}</div>
              <div className="text-[10px] opacity-50 leading-tight mt-0.5 truncate">{t("purchase.hint")}</div>
            </div>
            <div className="text-[10px] text-orange-500/60 shrink-0 group-hover:translate-x-0.5 transition-transform">→</div>
          </button>
        </>
      )}

      {/* Account mode */}
      {authMode === "account" && (
        <AccountLogin
          onConfigReady={handleAccountConfigReady}
          defaultModel={defaultModel}
          onModelChange={onModelChange}
          apiModels={apiModels}
          modelsLoading={modelsLoading}
          modelsError={modelsError}
        />
      )}
    </div>
  );
}
