import { invoke } from "@tauri-apps/api/core";
import type {
  ImportPreview,
  ModelList,
  ProviderInput,
  SettingsInput,
  Snapshot,
} from "./types";

function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(name, args);
}

export const api = {
  snapshot: () => command<Snapshot>("snapshot"),
  enable: () => command<Snapshot>("enable"),
  disable: () => command<Snapshot>("disable"),
  saveProvider: (input: ProviderInput) => command<Snapshot>("save_provider", { input }),
  deleteProvider: (providerId: string) =>
    command<Snapshot>("delete_provider", { providerId }),
  switchProvider: (providerId: string) =>
    command<Snapshot>("switch_provider", { providerId }),
  parseImportUrl: (url: string) => command<ImportPreview>("parse_import_url", { url }),
  importProvider: (url: string, switchNow: boolean) =>
    command<Snapshot>("import_provider", { url, switchNow }),
  fetchProviderModels: (providerId: string) =>
    command<ModelList>("fetch_provider_models", { providerId }),
  providerApiKey: (providerId: string) =>
    command<string>("provider_api_key", { providerId }),
  updateSettings: (input: SettingsInput) => command<Snapshot>("update_settings", { input }),
  selectCodexDirectory: () => command<string | null>("select_codex_directory"),
  restoreBackup: () => command<Snapshot>("restore_backup"),
};
