import type { KeyStatus, LoginMode } from "./types";
import type { MessageKey, TranslationParams } from "../i18n";

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
