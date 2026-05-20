import rawProfiles from "./site-profiles.json";

export type SwitchSiteKey = "hajimi" | "claude" | "xuanji";

export interface SwitchSiteProfile {
  siteKey: SwitchSiteKey;
  brandName: string;
  brandShortName: string;
  appName: string;
  appSubtitle: string;
  defaultBaseUrl: string;
  defaultModel: string;
  storeUrl: string;
  docsUrl: string;
  providerId: string;
  providerName: string;
  localStoragePrefix: string;
  legacyStoragePrefix: string | null;
  sqliteDirName: string;
  portableHomeDirName: string;
  exportFileName: string;
  bundleIdentifier: string;
  windowTitle: string;
  productName: string;
  iconSource: string;
  iconOutputDir: string;
  releaseAssetPrefix: string;
  iconGradient: [string, string, string];
}

type SiteProfileMap = Record<SwitchSiteKey, SwitchSiteProfile>;

const SITE_PROFILES = rawProfiles as SiteProfileMap;

const requestedSite = (import.meta.env.VITE_SWITCH_SITE || "hajimi") as SwitchSiteKey;

export const SITE_KEY: SwitchSiteKey =
  requestedSite in SITE_PROFILES ? requestedSite : "hajimi";

export const SITE_PROFILE = SITE_PROFILES[SITE_KEY];

type StorageSuffixMap = {
  theme: string;
  url: string;
  saveKey: string;
  key: string;
  model: string;
  cliModels: string;
  onboardingDone: string;
  language: string;
  tab: string;
  authMode: string;
  accountUrl: string;
  accountSession: string;
  accountUserId: string;
  accountUsername: string;
  syncLog: string;
};

const STORAGE_SUFFIXES: StorageSuffixMap = {
  theme: "theme",
  url: "url",
  saveKey: "save-key",
  key: "key",
  model: "model",
  cliModels: "cli-models",
  onboardingDone: "onboarding-done",
  language: "lang",
  tab: "tab",
  authMode: "auth-mode",
  accountUrl: "account-url",
  accountSession: "account-session",
  accountUserId: "account-user-id",
  accountUsername: "account-username",
  syncLog: "sync-log",
};

function makeStorageKeys(prefix: string): StorageSuffixMap {
  return Object.fromEntries(
    Object.entries(STORAGE_SUFFIXES).map(([key, suffix]) => [key, `${prefix}-${suffix}`])
  ) as StorageSuffixMap;
}

export const storageKeys = makeStorageKeys(SITE_PROFILE.localStoragePrefix);

const legacyStorageKeys = SITE_PROFILE.legacyStoragePrefix
  ? makeStorageKeys(SITE_PROFILE.legacyStoragePrefix)
  : null;

export function readSiteStorage(key: keyof StorageSuffixMap): string | null {
  const primary = localStorage.getItem(storageKeys[key]);
  if (primary !== null) {
    return primary;
  }

  if (!legacyStorageKeys) {
    return null;
  }

  const legacy = localStorage.getItem(legacyStorageKeys[key]);
  if (legacy !== null) {
    localStorage.setItem(storageKeys[key], legacy);
  }
  return legacy;
}

export function getOriginFromBaseUrl(baseUrl: string): string {
  try {
    return new URL(baseUrl).origin;
  } catch {
    return new URL(SITE_PROFILE.defaultBaseUrl).origin;
  }
}

export function getConsoleStoreUrl(baseUrl?: string): string {
  const origin = getOriginFromBaseUrl(baseUrl || SITE_PROFILE.defaultBaseUrl);
  return `${origin}/console/store`;
}

export const DEFAULT_URL = SITE_PROFILE.defaultBaseUrl;
export const DEFAULT_MODEL = SITE_PROFILE.defaultModel;
export const THEME_LIGHT = "switch-light";
export const THEME_DARK = "switch-dark";
