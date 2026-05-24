import React from "react";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  CheckCircle2,
  Download,
  ExternalLink,
  FolderCog,
  Globe2,
  Loader2,
  Settings,
  Wrench,
} from "lucide-react";
import type { RunAction } from "../app/types";
import { Page } from "../components/layout/Page";
import { Button, SegmentedControl, Switch, TextInput } from "../components/ui";
import { SettingRow, SettingSection } from "../features/settings/SettingSection";
import { api } from "../lib/api";
import { compactPath } from "../lib/format";
import type { LanguageMode, SettingsInput, Snapshot, ThemeMode, UpdateInfo } from "../lib/types";
import { useI18n } from "../i18n";

export function SettingsPage({
  snapshot,
  busy,
  run,
  onPreviewTheme,
}: {
  snapshot: Snapshot;
  busy: boolean;
  run: RunAction;
  onPreviewTheme: (mode: ThemeMode | null) => void;
}) {
  const [form, setForm] = React.useState<SettingsInput>({
    proxyPort: snapshot.state.proxyPort,
    codexDirOverride: snapshot.state.codexDirOverride ?? "",
    launchAtLogin: snapshot.state.launchAtLogin,
    themeMode: snapshot.state.themeMode,
    languageMode: snapshot.state.languageMode,
  });
  const [saveState, setSaveState] = React.useState<"idle" | "saving" | "saved" | "error">("idle");
  const [saveError, setSaveError] = React.useState<string | null>(null);
  const [updateState, setUpdateState] = React.useState<"idle" | "checking" | "error">("idle");
  const [updateMessage, setUpdateMessage] = React.useState<string | null>(null);
  const [updateInfo, setUpdateInfo] = React.useState<UpdateInfo | null>(null);
  const { t } = useI18n();
  const [appVersion, setAppVersion] = React.useState<string | null>(null);
  const [appVersionFailed, setAppVersionFailed] = React.useState(false);
  const saveTimerRef = React.useRef<number | null>(null);
  const formRef = React.useRef<SettingsInput>(form);

  React.useEffect(() => {
    const next = {
      proxyPort: snapshot.state.proxyPort,
      codexDirOverride: snapshot.state.codexDirOverride ?? "",
      launchAtLogin: snapshot.state.launchAtLogin,
      themeMode: snapshot.state.themeMode,
      languageMode: snapshot.state.languageMode,
    };
    setForm(next);
    formRef.current = next;
  }, [
    snapshot.state.proxyPort,
    snapshot.state.codexDirOverride,
    snapshot.state.launchAtLogin,
    snapshot.state.themeMode,
    snapshot.state.languageMode,
  ]);

  React.useEffect(() => {
    let cancelled = false;

    void getVersion()
      .then((version) => {
        if (!cancelled) {
          setAppVersion(version);
          setAppVersionFailed(false);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setAppVersionFailed(true);
        }
      });

    return () => {
      cancelled = true;
      if (saveTimerRef.current) {
        window.clearTimeout(saveTimerRef.current);
      }
      onPreviewTheme(null);
    };
  }, [onPreviewTheme]);

  const saveSettings = React.useCallback(
    async (input: SettingsInput) => {
      if (input.proxyPort <= 0 || !Number.isFinite(input.proxyPort)) {
        setSaveState("error");
        setSaveError(t("settings.portRequired"));
        return null;
      }

      setSaveState("saving");
      setSaveError(null);
      const next = await run(() => api.updateSettings(input));
      if (next) {
        formRef.current = {
          proxyPort: next.state.proxyPort,
          codexDirOverride: next.state.codexDirOverride ?? "",
          launchAtLogin: next.state.launchAtLogin,
          themeMode: next.state.themeMode,
          languageMode: next.state.languageMode,
        };
        setSaveState("saved");
        window.setTimeout(() => setSaveState("idle"), 1300);
        return next;
      }

      setSaveState("error");
      setSaveError(t("settings.saveFailedDetail"));
      onPreviewTheme(snapshot.state.themeMode);
      return null;
    },
    [onPreviewTheme, run, snapshot.state.themeMode, t],
  );

  const scheduleSave = React.useCallback(
    (input: SettingsInput, delay = 500) => {
      if (saveTimerRef.current) {
        window.clearTimeout(saveTimerRef.current);
      }
      saveTimerRef.current = window.setTimeout(() => {
        saveTimerRef.current = null;
        void saveSettings(input);
      }, delay);
    },
    [saveSettings],
  );

  const set = (
    patch: Partial<SettingsInput>,
    options?: { delay?: number; immediate?: boolean },
  ) => {
    const next = { ...formRef.current, ...patch };
    formRef.current = next;
    setForm(next);
    if (patch.themeMode) {
      onPreviewTheme(patch.themeMode);
    }
    if (options?.immediate) {
      if (saveTimerRef.current) {
        window.clearTimeout(saveTimerRef.current);
      }
      void saveSettings(next);
    } else {
      scheduleSave(next, options?.delay);
    }
  };

  const flushSave = () => {
    if (saveTimerRef.current) {
      window.clearTimeout(saveTimerRef.current);
      saveTimerRef.current = null;
    }
    void saveSettings(formRef.current);
  };

  const pickCodexDir = async () => {
    try {
      const selected = await api.selectCodexDirectory();
      if (!selected) {
        return;
      }
      set({ codexDirOverride: selected }, { immediate: true });
    } catch (error) {
      setSaveState("error");
      setSaveError(String(error));
    }
  };

  const checkForUpdates = async () => {
    try {
      setUpdateState("checking");
      setUpdateMessage(null);
      const info = await api.updateInfo();
      setUpdateInfo(info);
      setUpdateState("idle");
      setUpdateMessage(info.notes);
    } catch (error) {
      setUpdateState("error");
      setUpdateMessage(String(error));
    }
  };

  const openReleasePage = async () => {
    if (!updateInfo?.releaseUrl) {
      return;
    }

    try {
      await openUrl(updateInfo.releaseUrl);
    } catch (error) {
      setUpdateState("error");
      setUpdateMessage(t("settings.openDownloadFailed", { error: String(error) }));
    }
  };

  const displayedAppVersion = appVersion ?? (appVersionFailed ? t("app.unknown") : t("app.loading"));
  const updateDetail = updateMessage ?? t("settings.updateDetail");

  return (
    <Page
      title={t("settings.title")}
      actions={
        <div className="min-w-[96px] text-right text-[12px] font-semibold text-stone-500">
          {saveState === "saving" && (
            <span className="inline-flex items-center gap-1 text-stone-500">
              <Loader2 className="animate-spin" size={13} />
              {t("settings.saving")}
            </span>
          )}
          {saveState === "saved" && (
            <span className="inline-flex items-center gap-1 text-emerald-800">
              <CheckCircle2 size={13} />
              {t("settings.saved")}
            </span>
          )}
          {saveState === "error" && (
            <span
              className="inline-block max-w-[140px] truncate text-red-600"
              title={saveError ?? undefined}
            >
              {saveError ?? t("settings.saveFailed")}
            </span>
          )}
        </div>
      }
    >
      <SettingSection icon={<Globe2 size={20} />} title={t("settings.proxy")}>
        <SettingRow label={t("settings.port")}>
          <TextInput
            className="w-[132px]"
            type="number"
            value={String(form.proxyPort)}
            onChange={(event) => set({ proxyPort: Number(event.target.value) })}
            onBlur={flushSave}
          />
        </SettingRow>
        <SettingRow
          label={t("settings.codexDirectory")}
          detail={t("settings.configPath", { path: compactPath(snapshot.codex.configPath, 42) })}
        >
          <div className="flex min-w-0 items-center gap-2">
            <TextInput
              className="w-[180px]"
              value={form.codexDirOverride ?? ""}
              onChange={(event) => set({ codexDirOverride: event.target.value })}
              onBlur={flushSave}
              placeholder={snapshot.codex.codexDir}
            />
            <Button type="button" onClick={pickCodexDir} title={t("settings.chooseFolder")}>
              <FolderCog size={16} />
            </Button>
          </div>
        </SettingRow>
      </SettingSection>

      <SettingSection icon={<Settings size={20} />} title={t("settings.preferences")}>
        <SettingRow label={t("settings.launchAtLogin")}>
          <Switch
            checked={form.launchAtLogin}
            onChange={(event) =>
              set({ launchAtLogin: event.target.checked }, { immediate: true })
            }
          />
        </SettingRow>
        <SettingRow label={t("settings.theme")}>
          <SegmentedControl
            value={form.themeMode}
            options={[
              { label: t("settings.themeLight"), value: "light" },
              { label: t("settings.themeDark"), value: "dark" },
            ]}
            onChange={(themeMode) => set({ themeMode }, { immediate: true })}
          />
        </SettingRow>
        <SettingRow label={t("settings.language")}>
          <SegmentedControl<LanguageMode>
            value={form.languageMode}
            options={[
              { label: t("settings.languageSystem"), value: "system" },
              { label: t("settings.languageChinese"), value: "zh-CN" },
              { label: t("settings.languageEnglish"), value: "en-US" },
            ]}
            onChange={(languageMode) => set({ languageMode }, { immediate: true })}
          />
        </SettingRow>
      </SettingSection>

      <SettingSection icon={<Wrench size={20} />} title={t("settings.maintenance")}>
        <SettingRow label={t("settings.currentVersion")} detail={t("settings.appVersion")}>
          <div className="text-[13px] font-semibold text-stone-700">v{displayedAppVersion}</div>
        </SettingRow>
        <SettingRow
          label={t("settings.configBackup")}
          detail={
            snapshot.state.lastBackupPath
              ? compactPath(snapshot.state.lastBackupPath, 44)
              : t("settings.noBackup")
          }
        >
          <Button
            disabled={busy || !snapshot.state.lastBackupPath}
            onClick={() => run(api.restoreBackup, t("settings.restoredBackup"))}
          >
            <FolderCog size={16} />
            {t("settings.restoreBackup")}
          </Button>
        </SettingRow>
        <SettingRow label={t("settings.checkUpdates")} detail={updateDetail}>
          <div className="flex items-center gap-2">
            <Button
              disabled={busy || updateState === "checking"}
              onClick={checkForUpdates}
            >
              {updateState === "checking" ? (
                <Loader2 className="animate-spin" size={16} />
              ) : (
                <Download size={16} />
              )}
              {updateState === "checking" ? t("settings.checking") : t("settings.checkUpdates")}
            </Button>
            <Button
              disabled={!updateInfo?.hasUpdate}
              onClick={openReleasePage}
              title={updateInfo?.hasUpdate ? t("settings.openReleasePage") : t("settings.downloadNeedsUpdate")}
            >
              <ExternalLink size={16} />
              {t("settings.downloadNew")}
            </Button>
          </div>
        </SettingRow>
      </SettingSection>
    </Page>
  );
}
