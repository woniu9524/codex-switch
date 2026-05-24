import type { KeyStatus, LoginMode } from "./types";
import type { MessageKey, TranslationParams } from "../i18n";
import type { Locale } from "../i18n/types";

export function compactPath(value: string, max = 44) {
  if (value.length <= max) return value;
  const normalized = value.split("\\").join("/");
  const parts = normalized.split("/");
  if (parts.length <= 2) return `...${value.slice(-(max - 3))}`;
  const tail = parts.slice(-2).join("/");
  return `.../${tail}`;
}

type Translate = (key: MessageKey, params?: TranslationParams) => string;

export function loginModeLabel(mode: LoginMode, t: Translate) {
  return {
    api_key: "API Key",
    chat_gpt: t("format.loginChatGpt"),
    mixed: t("format.loginMixed"),
    unknown: t("format.loginUnknown"),
  }[mode];
}

export function compactNumber(value: number | null | undefined, locale: Locale = "en-US") {
  const number = Number(value ?? 0);
  if (!Number.isFinite(number)) return "0";
  const abs = Math.abs(number);

  if (abs >= 1_000_000_000) return `${formatCompactUnit(number / 1_000_000_000, locale)}B`;
  if (abs >= 10_000) return `${formatCompactUnit(number / 1_000_000, locale)}M`;

  return new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(number);
}

function formatCompactUnit(value: number, locale: Locale) {
  const abs = Math.abs(value);
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: abs >= 10 ? 1 : 2,
  }).format(value);
}

export function formatDecimal(value: number | null | undefined, digits = 1) {
  const number = Number(value ?? 0);
  if (!Number.isFinite(number)) return "0";
  return new Intl.NumberFormat(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: digits,
  }).format(number);
}

export function keyStatusLabel(status: KeyStatus | null | undefined, t: Translate) {
  if (status === "present") return t("format.keySaved");
  return t("format.keyMissing");
}

export function relativeTime(timestamp: number | null | undefined, t: Translate) {
  if (!timestamp) return t("format.noRequests");
  const diff = Date.now() - timestamp;
  if (diff < 60_000) return t("format.justNow");
  if (diff < 3_600_000) return t("format.minutesAgo", { count: Math.floor(diff / 60_000) });
  return new Date(timestamp).toLocaleString();
}
