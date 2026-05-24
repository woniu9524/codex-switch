export type KeyStatus = "present" | "missing";
export type LoginMode = "api_key" | "chat_gpt" | "mixed" | "unknown";
export type ThemeMode = "light" | "dark";

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

export interface RestorePoint {
  modelProvider?: string | null;
  model?: string | null;
  openaiBaseUrl?: string | null;
}

export interface StoredState {
  version: number;
  enabled: boolean;
  activeProviderId?: string | null;
  proxyPort: number;
  codexDirOverride?: string | null;
  launchAtLogin: boolean;
  themeMode: ThemeMode;
  lastBackupPath?: string | null;
  lastWrittenModel?: string | null;
  restorePoint?: RestorePoint | null;
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
