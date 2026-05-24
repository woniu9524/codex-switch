import React from "react";
import { CheckCircle2, FolderCog, Globe2, Loader2, Settings, Wrench } from "lucide-react";
import type { RunAction } from "../app/types";
import { Button, SegmentedControl, Switch, TextInput } from "../components/ui";
import { Page } from "../components/layout/Page";
import { SettingRow, SettingSection } from "../features/settings/SettingSection";
import { api } from "../lib/api";
import { compactPath } from "../lib/format";
import type { SettingsInput, Snapshot, ThemeMode } from "../lib/types";

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
  });
  const [saveState, setSaveState] = React.useState<"idle" | "saving" | "saved" | "error">("idle");
  const [saveError, setSaveError] = React.useState<string | null>(null);
  const saveTimerRef = React.useRef<number | null>(null);
  const formRef = React.useRef<SettingsInput>(form);

  React.useEffect(() => {
    const next = {
      proxyPort: snapshot.state.proxyPort,
      codexDirOverride: snapshot.state.codexDirOverride ?? "",
      launchAtLogin: snapshot.state.launchAtLogin,
      themeMode: snapshot.state.themeMode,
    };
    setForm(next);
    formRef.current = next;
  }, [
    snapshot.state.proxyPort,
    snapshot.state.codexDirOverride,
    snapshot.state.launchAtLogin,
    snapshot.state.themeMode,
  ]);

  React.useEffect(() => {
    return () => {
      if (saveTimerRef.current) window.clearTimeout(saveTimerRef.current);
      onPreviewTheme(null);
    };
  }, [onPreviewTheme]);

  const saveSettings = React.useCallback(
    async (input: SettingsInput) => {
      if (input.proxyPort <= 0 || !Number.isFinite(input.proxyPort)) {
        setSaveState("error");
        setSaveError("端口必须大于 0");
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
        };
        setSaveState("saved");
        window.setTimeout(() => setSaveState("idle"), 1300);
        return next;
      }
      setSaveState("error");
      setSaveError("保存失败，请检查输入");
      onPreviewTheme(snapshot.state.themeMode);
      return null;
    },
    [onPreviewTheme, run, snapshot.state.themeMode],
  );

  const scheduleSave = React.useCallback(
    (input: SettingsInput, delay = 500) => {
      if (saveTimerRef.current) window.clearTimeout(saveTimerRef.current);
      saveTimerRef.current = window.setTimeout(() => {
        saveTimerRef.current = null;
        void saveSettings(input);
      }, delay);
    },
    [saveSettings],
  );

  const set = (patch: Partial<SettingsInput>, options?: { delay?: number; immediate?: boolean }) => {
    const next = { ...formRef.current, ...patch };
    formRef.current = next;
    setForm(next);
    if (patch.themeMode) onPreviewTheme(patch.themeMode);
    if (options?.immediate) {
      if (saveTimerRef.current) window.clearTimeout(saveTimerRef.current);
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
      if (!selected) return;
      set({ codexDirOverride: selected }, { immediate: true });
    } catch (error) {
      setSaveState("error");
      setSaveError(String(error));
    }
  };

  return (
    <Page
      title="设置"
      actions={
        <div className="min-w-[82px] text-right text-[12px] font-semibold text-stone-500">
          {saveState === "saving" && (
            <span className="inline-flex items-center gap-1 text-stone-500">
              <Loader2 className="animate-spin" size={13} />
              保存中
            </span>
          )}
          {saveState === "saved" && (
            <span className="inline-flex items-center gap-1 text-emerald-800">
              <CheckCircle2 size={13} />
              已保存
            </span>
          )}
          {saveState === "error" && (
            <span className="inline-block max-w-[130px] truncate text-red-600" title={saveError ?? undefined}>
              {saveError ?? "保存失败"}
            </span>
          )}
        </div>
      }
    >
      <SettingSection icon={<Globe2 size={20} />} title="代理">
        <SettingRow label="端口">
          <TextInput
            className="w-[132px]"
            type="number"
            value={String(form.proxyPort)}
            onChange={(event) => set({ proxyPort: Number(event.target.value) })}
            onBlur={flushSave}
          />
        </SettingRow>
        <SettingRow label="Codex 目录" detail={`配置路径 ${compactPath(snapshot.codex.configPath, 42)}`}>
          <div className="flex min-w-0 items-center gap-2">
            <TextInput
              className="w-[180px]"
              value={form.codexDirOverride ?? ""}
              onChange={(event) => set({ codexDirOverride: event.target.value })}
              onBlur={flushSave}
              placeholder={snapshot.codex.codexDir}
            />
            <Button type="button" onClick={pickCodexDir} title="选择文件夹">
              <FolderCog size={16} />
            </Button>
          </div>
        </SettingRow>
      </SettingSection>

      <SettingSection icon={<Settings size={20} />} title="偏好">
        <SettingRow label="开机启动">
          <Switch
            checked={form.launchAtLogin}
            onChange={(event) => set({ launchAtLogin: event.target.checked }, { immediate: true })}
          />
        </SettingRow>
        <SettingRow label="主题">
          <SegmentedControl
            value={form.themeMode}
            options={[
              { label: "浅色", value: "light" },
              { label: "深色", value: "dark" },
            ]}
            onChange={(themeMode) => set({ themeMode }, { immediate: true })}
          />
        </SettingRow>
      </SettingSection>

      <SettingSection icon={<Wrench size={20} />} title="维护">
        <SettingRow label="配置备份" detail={snapshot.state.lastBackupPath ? compactPath(snapshot.state.lastBackupPath, 44) : "暂无备份"}>
          <Button
            disabled={busy || !snapshot.state.lastBackupPath}
            onClick={() => run(api.restoreBackup, "已恢复备份。")}
          >
            <FolderCog size={16} />
            恢复
          </Button>
        </SettingRow>
      </SettingSection>
    </Page>
  );
}
