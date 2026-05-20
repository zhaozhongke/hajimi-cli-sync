import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./locales/en.json";
import zh from "./locales/zh.json";
import { readSiteStorage, SITE_PROFILE, storageKeys } from "./site";

const savedLang = readSiteStorage("language");
const defaultLang = navigator.language.startsWith("zh") ? "zh" : "en";

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    zh: { translation: zh },
  },
  lng: savedLang || defaultLang,
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
    defaultVariables: {
      appName: SITE_PROFILE.appName,
      appSubtitle: SITE_PROFILE.appSubtitle,
      brandName: SITE_PROFILE.brandName,
      brandShortName: SITE_PROFILE.brandShortName,
      providerName: SITE_PROFILE.providerName,
      baseUrlExample: SITE_PROFILE.defaultBaseUrl,
    },
  },
});

if (!savedLang) {
  localStorage.setItem(storageKeys.language, defaultLang);
}

export default i18n;
