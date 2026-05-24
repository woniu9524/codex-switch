import React from "react";
import { enUS } from "./messages.en-US";
import { zhCN, type MessageKey } from "./messages.zh-CN";
import type { LanguageMode, Locale, TranslationParams } from "./types";

const messages = {
  "zh-CN": zhCN,
  "en-US": enUS,
} satisfies Record<Locale, Record<MessageKey, string>>;

type I18nContextValue = {
  locale: Locale;
  languageMode: LanguageMode;
  t: (key: MessageKey, params?: TranslationParams) => string;
};

const I18nContext = React.createContext<I18nContextValue | null>(null);

export function resolveLocale(languageMode: LanguageMode, systemLanguage = navigator.language): Locale {
  if (languageMode !== "system") return languageMode;
  return systemLanguage.toLowerCase().startsWith("zh") ? "zh-CN" : "en-US";
}

export function I18nProvider({
  languageMode,
  children,
}: {
  languageMode: LanguageMode;
  children: React.ReactNode;
}) {
  const [systemLanguage, setSystemLanguage] = React.useState(() => navigator.language);

  React.useEffect(() => {
    const languages = navigator.languages ?? [navigator.language];
    setSystemLanguage(languages[0] ?? navigator.language);
  }, []);

  const locale = resolveLocale(languageMode, systemLanguage);

  React.useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const value = React.useMemo<I18nContextValue>(
    () => ({
      locale,
      languageMode,
      t: (key, params) => interpolate(messages[locale][key], params),
    }),
    [languageMode, locale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const context = React.useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used within I18nProvider");
  }
  return context;
}

function interpolate(template: string, params?: TranslationParams) {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, key) => {
    const value = params[key];
    return value === undefined ? match : String(value);
  });
}

export type { LanguageMode, Locale, MessageKey, TranslationParams };
