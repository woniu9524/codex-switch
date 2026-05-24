export type KeyStatus = "present" | "missing";
export type LoginMode = "api_key" | "chat_gpt" | "mixed" | "unknown";
export type ThemeMode = "light" | "dark";
export type Locale = "zh-CN" | "en-US";
export type LanguageMode = "system" | Locale;

export interface Provider {
  id: string;
  name: string;
  endpoint: string;
  apiKeyRef?: string | null;
  model: string;
  homepage?: string | null;
  notes?: string | null;
  icon?: string | null;
  disableImageGeneration: boolean;
  createdAt: number;
  updatedAt: number;
  keyStatus: KeyStatus;
}

export type ConfigLeaseStatus = "pending" | "applied";

export interface OriginalCodexConfig {
  modelProvider?: string | null;
  model?: string | null;
  openaiBaseUrl?: string | null;
  switchProviderTable?: string | null;
}

export interface ConfigLease {
  codexDirOverride?: string | null;
  backupPath: string;
  originalConfig: OriginalCodexConfig;
  lastWrittenModel: string;
  proxyPort: number;
  status: ConfigLeaseStatus;
}

export interface StoredState {
  version: number;
  enabled: boolean;
  activeProviderId?: string | null;
  proxyPort: number;
  codexDirOverride?: string | null;
  launchAtLogin: boolean;
  themeMode: ThemeMode;
  languageMode: LanguageMode;
  lastBackupPath?: string | null;
  configLease?: ConfigLease | null;
  lastWrittenModel?: string | null;
  providers: Provider[];
}

export interface RequestEvent {
  timestamp: number;
  providerId: string;
  method: string;
  path: string;
  status: number;
  durationMs: number;
  error?: string | null;
}

export interface CodexDiagnostic {
  codexDir: string;
  configPath: string;
  authPath: string;
  configExists: boolean;
  authExists: boolean;
  loginMode: LoginMode;
  detail: string;
}

export interface RuntimePaths {
  appHome: string;
  stateFile: string;
  backupDir: string;
  logDir: string;
}

export interface Snapshot {
  state: StoredState;
  activeProvider?: Provider | null;
  proxyRunning: boolean;
  codex: CodexDiagnostic;
  paths: RuntimePaths;
  recentLogs: RequestEvent[];
}

export interface ProviderInput {
  id?: string | null;
  name: string;
  endpoint: string;
  apiKey?: string | null;
  model: string;
  homepage?: string | null;
  notes?: string | null;
  icon?: string | null;
  disableImageGeneration: boolean;
  switchNow: boolean;
}

export interface SettingsInput {
  proxyPort: number;
  codexDirOverride?: string | null;
  launchAtLogin: boolean;
  themeMode: ThemeMode;
  languageMode: LanguageMode;
}

export interface ImportPreview {
  scheme: string;
  source: string;
  name: string;
  endpoint: string;
  apiKeyMasked?: string | null;
  hasApiKey: boolean;
  model: string;
  homepage?: string | null;
  notes?: string | null;
  icon?: string | null;
  enabledHint: boolean;
}

export interface ModelList {
  providerId: string;
  models: string[];
}

export interface UpdateInfo {
  currentVersion: string;
  latestVersion?: string | null;
  hasUpdate: boolean;
  releaseUrl: string;
  checkedAt: number;
  notes: string;
}
