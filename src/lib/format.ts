import type { KeyStatus, LoginMode } from "./types";

export function compactPath(value: string, max = 44) {
  if (value.length <= max) return value;
  const normalized = value.split("\\").join("/");
  const parts = normalized.split("/");
  if (parts.length <= 2) return `...${value.slice(-(max - 3))}`;
  const tail = parts.slice(-2).join("/");
  return `.../${tail}`;
}

export function loginModeLabel(mode: LoginMode) {
  return {
    api_key: "API Key",
    chat_gpt: "ChatGPT 登录",
    mixed: "混合登录",
    unknown: "未确认",
  }[mode];
}

export function keyStatusLabel(status?: KeyStatus | null) {
  if (status === "present") return "Key 已保存";
  return "缺少 Key";
}

export function relativeTime(timestamp?: number | null) {
  if (!timestamp) return "无请求";
  const diff = Date.now() - timestamp;
  if (diff < 60_000) return "刚刚";
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} 分钟前`;
  return new Date(timestamp).toLocaleString();
}
