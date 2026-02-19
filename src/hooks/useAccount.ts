import { invoke } from "@tauri-apps/api/core";
import { useState, useCallback } from "react";
import type { PlatformInfo, AccountInfo, ApiTokenInfo } from "../types";

const SESSION_KEYS = {
  mode: "hajimi-auth-mode",
  url: "hajimi-account-url",
  session: "hajimi-account-session",
  userId: "hajimi-account-user-id",
  username: "hajimi-account-username",
} as const;

export function useAccount() {
  const [platformInfo, setPlatformInfo] = useState<PlatformInfo | null>(null);
  const [accountInfo, setAccountInfo] = useState<AccountInfo | null>(null);
  const [tokens, setTokens] = useState<ApiTokenInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkPlatform = useCallback(async (baseUrl: string) => {
    setError(null);
    try {
      const info = await invoke<PlatformInfo>("check_platform", { baseUrl });
      setPlatformInfo(info);
      return info;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      setPlatformInfo(null);
      return null;
    }
  }, []);

  const login = useCallback(async (baseUrl: string, username: string, password: string) => {
    setLoading(true);
    setError(null);
    try {
      const info = await invoke<AccountInfo>("account_login", {
        baseUrl,
        username,
        password,
      });
      setAccountInfo(info);
      // Persist session to localStorage
      localStorage.setItem(SESSION_KEYS.url, baseUrl);
      localStorage.setItem(SESSION_KEYS.userId, String(info.user_id));
      localStorage.setItem(SESSION_KEYS.username, info.username);
      return info;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchTokens = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await invoke<ApiTokenInfo[]>("account_get_tokens");
      setTokens(list);
      return list;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg === "SESSION_EXPIRED") {
        setAccountInfo(null);
        clearSession();
      }
      setError(msg);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const checkSession = useCallback(async () => {
    // Try to restore session from localStorage
    const savedUrl = localStorage.getItem(SESSION_KEYS.url);
    const savedUserId = localStorage.getItem(SESSION_KEYS.userId);
    const savedUsername = localStorage.getItem(SESSION_KEYS.username);
    const savedSession = localStorage.getItem(SESSION_KEYS.session);

    if (!savedUrl || !savedUserId || !savedUsername || !savedSession) {
      return null;
    }

    try {
      // Restore session into Rust state
      await invoke("account_restore_session", {
        baseUrl: savedUrl,
        sessionCookie: savedSession,
        userId: Number(savedUserId),
        username: savedUsername,
      });

      // Verify it's still valid
      const info = await invoke<AccountInfo>("account_check_session");
      setAccountInfo(info);
      return info;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg === "SESSION_EXPIRED" || msg === "NOT_LOGGED_IN") {
        clearSession();
      }
      return null;
    }
  }, []);

  const saveSessionCookie = useCallback((cookie: string) => {
    localStorage.setItem(SESSION_KEYS.session, cookie);
  }, []);

  const logout = useCallback(async () => {
    try {
      await invoke("account_logout");
    } catch {
      // Ignore logout errors
    }
    setAccountInfo(null);
    setTokens([]);
    setPlatformInfo(null);
    clearSession();
  }, []);

  return {
    platformInfo,
    accountInfo,
    tokens,
    loading,
    error,
    checkPlatform,
    login,
    fetchTokens,
    checkSession,
    saveSessionCookie,
    logout,
    setError,
  };
}

function clearSession() {
  localStorage.removeItem(SESSION_KEYS.session);
  localStorage.removeItem(SESSION_KEYS.userId);
  localStorage.removeItem(SESSION_KEYS.username);
  localStorage.removeItem(SESSION_KEYS.url);
}
