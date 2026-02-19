import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { LogIn, RefreshCw, LogOut, ExternalLink, Check, AlertCircle } from "lucide-react";
import { toast } from "sonner";
import { useAccount } from "../hooks/useAccount";
import { ModelSelector } from "./ModelSelector";
import type { ApiTokenInfo } from "../types";

interface AccountLoginProps {
  onConfigReady: (url: string, apiKey: string) => void;
  defaultModel: string;
  onModelChange: (model: string) => void;
  apiModels: string[];
  modelsLoading: boolean;
  modelsError: string | null;
}

export function AccountLogin({
  onConfigReady,
  defaultModel,
  onModelChange,
  apiModels,
  modelsLoading,
  modelsError,
}: AccountLoginProps) {
  const { t } = useTranslation();
  const {
    platformInfo,
    accountInfo,
    tokens,
    loading,
    error,
    checkPlatform,
    login,
    fetchTokens,
    checkSession,
    logout,
    setError,
  } = useAccount();

  const [platformUrl, setPlatformUrl] = useState(
    () => localStorage.getItem("hajimi-account-url") || "https://vip.aipro.love"
  );
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [selectedTokenId, setSelectedTokenId] = useState<number | null>(null);
  const [sessionChecked, setSessionChecked] = useState(false);

  // Check platform info when URL changes (debounced)
  useEffect(() => {
    if (!platformUrl.trim()) return;
    const timer = setTimeout(() => {
      checkPlatform(platformUrl);
    }, 600);
    return () => clearTimeout(timer);
  }, [platformUrl, checkPlatform]);

  // On mount: try restoring session
  useEffect(() => {
    const restore = async () => {
      const info = await checkSession();
      setSessionChecked(true);
      if (info) {
        // Session is valid, fetch tokens
        fetchTokens();
      }
    };
    restore();
  }, []);

  const handleLogin = async () => {
    if (!username.trim() || !password.trim()) return;
    setError(null);
    const info = await login(platformUrl, username.trim(), password.trim());
    if (info) {
      toast.success(t("account.loginSuccess", { name: info.display_name }));
      setPassword("");
      fetchTokens();
    }
  };

  const handleLogout = async () => {
    await logout();
    setSelectedTokenId(null);
    toast.success(t("account.logoutSuccess"));
  };

  const handleSelectToken = (token: ApiTokenInfo) => {
    setSelectedTokenId(token.id);
    // Apply to parent: set apiKey and url
    onConfigReady(platformUrl, token.key);
    toast.success(t("account.tokenSelected", { name: token.name }));
  };

  const handleRegister = () => {
    const registerUrl = `${platformUrl.replace(/\/$/, "")}/register`;
    invoke("open_external_url", { url: registerUrl });
  };

  const formatQuota = (quota: number): string => {
    if (quota >= 1_000_000) return `${(quota / 1_000_000).toFixed(1)}M`;
    if (quota >= 1_000) return `${(quota / 1_000).toFixed(1)}K`;
    return String(quota);
  };

  const formatExpiry = (ts: number): string => {
    if (ts === -1) return t("account.neverExpire");
    const date = new Date(ts * 1000);
    const now = new Date();
    if (date < now) return t("account.expired");
    return date.toLocaleDateString();
  };

  const getStatusBadge = (token: ApiTokenInfo) => {
    if (token.status === 2) return { text: t("account.statusDisabled"), cls: "badge-error" };
    if (token.status === 3) return { text: t("account.statusExpired"), cls: "badge-warning" };
    if (token.status === 4) return { text: t("account.statusExhausted"), cls: "badge-warning" };
    if (token.expired_time !== -1 && token.expired_time * 1000 < Date.now()) {
      return { text: t("account.statusExpired"), cls: "badge-warning" };
    }
    if (!token.unlimited_quota && token.remain_quota <= 0) {
      return { text: t("account.statusExhausted"), cls: "badge-warning" };
    }
    return { text: t("account.statusActive"), cls: "badge-success" };
  };

  const isTokenUsable = (token: ApiTokenInfo) => {
    if (token.status !== 1) return false;
    if (token.expired_time !== -1 && token.expired_time * 1000 < Date.now()) return false;
    if (!token.unlimited_quota && token.remain_quota <= 0) return false;
    return true;
  };

  // Sort: usable first, then by remain_quota desc
  const sortedTokens = useMemo(() => {
    return [...tokens].sort((a, b) => {
      const aUsable = isTokenUsable(a) ? 1 : 0;
      const bUsable = isTokenUsable(b) ? 1 : 0;
      if (aUsable !== bUsable) return bUsable - aUsable;
      return b.remain_quota - a.remain_quota;
    });
  }, [tokens]);

  // Not logged in — show login form
  if (!accountInfo) {
    // Don't show anything until session check is done
    if (!sessionChecked) {
      return (
        <div className="flex items-center justify-center py-6">
          <span className="loading loading-spinner loading-sm" />
        </div>
      );
    }

    return (
      <div className="space-y-3">
        {/* Platform URL */}
        <div className="form-control">
          <div className="flex items-center gap-2">
            <input
              type="text"
              className="input input-bordered input-sm w-full"
              value={platformUrl}
              onChange={(e) => setPlatformUrl(e.target.value)}
              placeholder={t("account.platformUrlHint")}
            />
            {platformInfo && (
              <span className="badge badge-ghost badge-sm whitespace-nowrap shrink-0">
                {platformInfo.system_name}
              </span>
            )}
          </div>
        </div>

        {/* Login form */}
        <div className="form-control">
          <input
            type="text"
            className="input input-bordered input-sm w-full"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder={t("account.username")}
            onKeyDown={(e) => e.key === "Enter" && handleLogin()}
          />
        </div>
        <div className="form-control">
          <input
            type="password"
            className="input input-bordered input-sm w-full"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder={t("account.password")}
            onKeyDown={(e) => e.key === "Enter" && handleLogin()}
          />
        </div>

        {/* Error */}
        {error && (
          <div className="flex items-center gap-1.5 text-error text-xs">
            <AlertCircle className="w-3.5 h-3.5 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {/* Login button */}
        <button
          className="btn btn-primary btn-sm w-full gap-1.5"
          onClick={handleLogin}
          disabled={loading || !username.trim() || !password.trim()}
        >
          {loading ? (
            <span className="loading loading-spinner loading-xs" />
          ) : (
            <LogIn className="w-3.5 h-3.5" />
          )}
          {loading ? t("account.loggingIn") : t("account.login")}
        </button>

        {/* Register link */}
        <div className="text-center">
          <button
            className="btn btn-ghost btn-xs gap-1 opacity-60"
            onClick={handleRegister}
          >
            {t("account.noAccount")}
            <ExternalLink className="w-3 h-3" />
          </button>
        </div>
      </div>
    );
  }

  // Logged in — show token list
  return (
    <div className="space-y-3">
      {/* User info bar */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 rounded-full bg-primary/20 flex items-center justify-center text-primary text-xs font-bold">
            {accountInfo.display_name.charAt(0).toUpperCase()}
          </div>
          <span className="text-sm font-medium">{accountInfo.display_name}</span>
          {platformInfo && (
            <span className="badge badge-ghost badge-xs">{platformInfo.system_name}</span>
          )}
        </div>
        <div className="flex items-center gap-1">
          <button
            className="btn btn-ghost btn-xs gap-1"
            onClick={fetchTokens}
            disabled={loading}
          >
            <RefreshCw className={`w-3 h-3 ${loading ? "animate-spin" : ""}`} />
          </button>
          <button
            className="btn btn-ghost btn-xs gap-1 text-error"
            onClick={handleLogout}
          >
            <LogOut className="w-3 h-3" />
          </button>
        </div>
      </div>

      {/* Token list */}
      {tokens.length === 0 && !loading && (
        <div className="text-center text-xs opacity-50 py-4">
          {t("account.noTokens")}
        </div>
      )}

      {loading && tokens.length === 0 && (
        <div className="flex items-center justify-center py-4">
          <span className="loading loading-spinner loading-sm" />
        </div>
      )}

      <div className="space-y-2 max-h-64 overflow-y-auto">
        {sortedTokens.map((token) => {
          const usable = isTokenUsable(token);
          const badge = getStatusBadge(token);
          const isSelected = selectedTokenId === token.id;
          const totalQuota = token.used_quota + token.remain_quota;
          const usagePercent = totalQuota > 0 ? (token.used_quota / totalQuota) * 100 : 0;

          return (
            <div
              key={token.id}
              className={`card bg-base-100 border transition-all ${
                isSelected
                  ? "border-primary shadow-sm"
                  : usable
                  ? "border-base-300 hover:border-base-content/20"
                  : "border-base-300 opacity-50"
              }`}
            >
              <div className="card-body p-3 gap-1.5">
                {/* Name + status */}
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium truncate">{token.name || `Token #${token.id}`}</span>
                  <span className={`badge badge-xs ${badge.cls}`}>{badge.text}</span>
                </div>

                {/* Key preview */}
                <div className="font-mono text-xs opacity-40 truncate">
                  {token.key.slice(0, 8)}...{token.key.slice(-6)}
                </div>

                {/* Quota */}
                {token.unlimited_quota ? (
                  <div className="text-xs opacity-60">{t("account.unlimitedQuota")}</div>
                ) : (
                  <div className="space-y-0.5">
                    <div className="flex justify-between text-xs opacity-60">
                      <span>
                        {t("account.used")}: {formatQuota(token.used_quota)}
                      </span>
                      <span>
                        {t("account.remaining")}: {formatQuota(token.remain_quota)}
                      </span>
                    </div>
                    <progress
                      className={`progress w-full h-1.5 ${usagePercent > 90 ? "progress-error" : "progress-primary"}`}
                      value={token.remain_quota}
                      max={totalQuota || 1}
                    />
                  </div>
                )}

                {/* Expiry + select button */}
                <div className="flex items-center justify-between mt-0.5">
                  <span className="text-xs opacity-40">
                    {t("account.expires")}: {formatExpiry(token.expired_time)}
                  </span>
                  {usable && (
                    <button
                      className={`btn btn-xs gap-1 ${isSelected ? "btn-primary" : "btn-ghost"}`}
                      onClick={() => handleSelectToken(token)}
                    >
                      {isSelected ? (
                        <>
                          <Check className="w-3 h-3" />
                          {t("account.selected")}
                        </>
                      ) : (
                        t("account.selectKey")
                      )}
                    </button>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* Model selector (same as manual mode step 3) */}
      {selectedTokenId && (
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
      )}

      {/* Error */}
      {error && (
        <div className="flex items-center gap-1.5 text-error text-xs">
          <AlertCircle className="w-3.5 h-3.5 shrink-0" />
          <span>{error === "SESSION_EXPIRED" ? t("account.sessionExpired") : error}</span>
        </div>
      )}
    </div>
  );
}
