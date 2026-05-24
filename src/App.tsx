import React from "react";
import { AlertCircle, Loader2, X } from "lucide-react";
import type { View } from "./app/types";
import { BottomNav } from "./components/layout/BottomNav";
import { TitleBar } from "./components/layout/TitleBar";
import { ControlPage } from "./pages/ControlPage";
import { ImportProviderPage } from "./pages/ImportProviderPage";
import { ProviderFormPage } from "./pages/ProviderFormPage";
import { ProvidersPage } from "./pages/ProvidersPage";
import { SettingsPage } from "./pages/SettingsPage";
import { api } from "./lib/api";
import type { Snapshot, ThemeMode } from "./lib/types";
import { cx } from "./lib/ui";

export function App() {
  const [view, setView] = React.useState<View>("control");
  const [editingProviderId, setEditingProviderId] = React.useState<string | null>(null);
  const [snapshot, setSnapshot] = React.useState<Snapshot | null>(null);
  const [busy, setBusy] = React.useState(false);
  const [message, setMessage] = React.useState<string | null>(null);
  const [themePreview, setThemePreview] = React.useState<ThemeMode | null>(null);
  const mainRef = React.useRef<HTMLElement | null>(null);

  const refresh = React.useCallback(async () => {
    try {
      setSnapshot(await api.snapshot());
    } catch (error) {
      setMessage(String(error));
    }
  }, []);

  React.useEffect(() => {
    void refresh();
  }, [refresh]);

  React.useEffect(() => {
    mainRef.current?.scrollTo({ top: 0 });
  }, [view]);

  const run = async (action: () => Promise<Snapshot>, success?: string) => {
    setBusy(true);
    setMessage(null);
    try {
      const next = await action();
      setSnapshot(next);
      if (success) setMessage(success);
      return next;
    } catch (error) {
      setMessage(String(error));
      return null;
    } finally {
      setBusy(false);
    }
  };

  const go = (next: View) => {
    setMessage(null);
    setEditingProviderId(null);
    setView(next);
  };

  const editProvider = (providerId: string) => {
    setMessage(null);
    setEditingProviderId(providerId);
    setView("provider-form");
  };

  const themeMode = themePreview ?? snapshot?.state.themeMode ?? "light";
  const previewTheme = React.useCallback((mode: ThemeMode | null) => {
    setThemePreview(mode);
  }, []);

  React.useEffect(() => {
    if (snapshot && themePreview === snapshot.state.themeMode) setThemePreview(null);
  }, [snapshot, themePreview]);

  return (
    <div
      className={cx(
        "app-shell flex h-screen min-h-0 flex-col overflow-hidden bg-stone-50 text-stone-950",
        themeMode === "dark" && "theme-dark",
      )}
    >
      <TitleBar
        online={Boolean(snapshot?.proxyRunning)}
        themeMode={themeMode}
      />

      <main ref={mainRef} className="min-h-0 flex-1 overflow-y-auto px-4 py-4">
        <div className="mx-auto flex min-h-full w-full max-w-[560px] flex-col gap-3">
          {message && (
            <div className="flex items-start gap-2 rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-[13px] font-medium leading-5 text-amber-800">
              <AlertCircle size={17} className="mt-0.5 shrink-0" />
              <span className="min-w-0 break-words">{message}</span>
              <button
                className="ml-auto grid size-5 shrink-0 place-items-center rounded text-amber-800 transition hover:bg-amber-100"
                onClick={() => setMessage(null)}
                title="关闭提示"
              >
                <X size={14} />
              </button>
            </div>
          )}

          {!snapshot ? (
            <div className="grid min-h-[260px] place-items-center text-stone-400">
              <Loader2 className="animate-spin" size={26} />
            </div>
          ) : (
            <>
              {view === "control" && (
                <ControlPage snapshot={snapshot} busy={busy} run={run} go={go} />
              )}
              {view === "providers" && (
                <ProvidersPage
                  snapshot={snapshot}
                  busy={busy}
                  run={run}
                  go={go}
                  editProvider={editProvider}
                />
              )}
              {view === "provider-form" && (
                <ProviderFormPage
                  snapshot={snapshot}
                  busy={busy}
                  run={run}
                  go={go}
                  providerId={editingProviderId}
                />
              )}
              {view === "provider-import" && (
                <ImportProviderPage snapshot={snapshot} busy={busy} run={run} go={go} />
              )}
              {view === "settings" && (
                <SettingsPage
                  snapshot={snapshot}
                  busy={busy}
                  run={run}
                  onPreviewTheme={previewTheme}
                />
              )}
            </>
          )}
        </div>
      </main>

      <BottomNav view={view} go={go} />
    </div>
  );
}
