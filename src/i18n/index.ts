import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zhCN from "./locales/zh-CN.json";
import enUS from "./locales/en-US.json";

export const SUPPORTED_LANGUAGES = ["zh-CN", "en-US"] as const;
export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

const STORAGE_KEY = "uniclipboard.language";

export function normalizeLanguage(
  language: string | null | undefined
): SupportedLanguage {
  if (!language) return "zh-CN";
  const lower = language.toLowerCase();
  if (lower.startsWith("zh")) return "zh-CN";
  return "en-US";
}

export function getInitialLanguage(): SupportedLanguage {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "zh-CN" || stored === "en-US") return stored;
  return normalizeLanguage(navigator.language);
}

export function persistLanguage(language: SupportedLanguage) {
  localStorage.setItem(STORAGE_KEY, language);
}

i18n.use(initReactI18next).init({
  resources: {
    "zh-CN": { translation: zhCN },
    "en-US": { translation: enUS },
  },
  lng: getInitialLanguage(),
  fallbackLng: "zh-CN",
  interpolation: { escapeValue: false },
});

persistLanguage(i18n.language as SupportedLanguage);

export default i18n;

