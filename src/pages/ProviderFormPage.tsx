import React from "react";
import { ChevronDown, Loader2, Save } from "lucide-react";
import type { RunAction, View } from "../app/types";
import { Button, Card, EmptyState, Field, Switch } from "../components/ui";
import { Page } from "../components/layout/Page";
import { ModelField } from "../features/providers/ModelField";
import { newProviderForm, providerToForm } from "../features/providers/providerForm";
import { api } from "../lib/api";
import type { Provider, ProviderInput, Snapshot } from "../lib/types";
import { useI18n } from "../i18n";
import type { ModelStatusTone } from "../features/providers/ModelField";

export function ProviderFormPage({
  snapshot,
  busy,
  run,
  go,
  providerId,
}: {
  snapshot: Snapshot;
  busy: boolean;
  run: RunAction;
  go: (view: View) => void;
  providerId: string | null;
}) {
  const editingProvider = providerId
    ? snapshot.state.providers.find((provider) => provider.id === providerId) ?? null
    : null;
  const initialForm = React.useMemo(
    () =>
      editingProvider
        ? providerToForm(editingProvider, snapshot.state.activeProviderId)
        : newProviderForm(snapshot.state.activeProviderId),
    [editingProvider, snapshot.state.activeProviderId],
  );
  const [form, setForm] = React.useState<ProviderInput>(initialForm);
  const [advancedOpen, setAdvancedOpen] = React.useState(false);
  const [modelOptions, setModelOptions] = React.useState<string[]>([]);
  const [modelLoading, setModelLoading] = React.useState(false);
  const [modelStatus, setModelStatus] = React.useState<string | null>(null);
  const [modelStatusTone, setModelStatusTone] = React.useState<ModelStatusTone>("success");
  const { t } = useI18n();

  React.useEffect(() => {
    setForm(initialForm);
    setModelOptions([]);
    setModelStatus(null);
    setModelStatusTone("success");
  }, [initialForm]);

  React.useEffect(() => {
    if (!editingProvider || editingProvider.keyStatus === "missing") return;
    let cancelled = false;
    api
      .providerApiKey(editingProvider.id)
      .then((apiKey) => {
        if (!cancelled) setForm((current) => ({ ...current, apiKey }));
      })
      .catch(() => {
        // Key loading failure should not block editing provider metadata.
      });
    return () => {
      cancelled = true;
    };
  }, [editingProvider]);

  const set = (patch: Partial<ProviderInput>) => setForm((value) => ({ ...value, ...patch }));

  const applyModels = React.useCallback((models: string[]) => {
    setModelOptions(models);
    if (models.length === 0) return;
    setForm((current) => ({
      ...current,
      model: models.includes(current.model) ? current.model : models[0],
    }));
  }, []);

  const loadModels = React.useCallback(
    async (provider: Provider, quiet = false) => {
      if (provider.keyStatus === "missing") return null;
      setModelLoading(true);
      if (!quiet) setModelStatus(null);
      try {
        const result = await api.fetchProviderModels(provider.id);
        applyModels(result.models);
        setModelStatus(t("providerForm.modelsFetched", { count: result.models.length }));
        setModelStatusTone("success");
        return result.models;
      } catch (error) {
        if (!quiet) {
          setModelStatus(t("providerForm.modelsFetchFailed", { error: String(error) }));
          setModelStatusTone("warning");
        }
        return null;
      } finally {
        setModelLoading(false);
      }
    },
    [applyModels, t],
  );

  React.useEffect(() => {
    if (!editingProvider) return;
    void loadModels(editingProvider, true);
  }, [editingProvider, loadModels]);

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    const canSwitchAfterSave =
      Boolean(form.apiKey?.trim()) ||
      Boolean(editingProvider && editingProvider.keyStatus !== "missing");
    const submission = {
      ...form,
      switchNow: form.switchNow && canSwitchAfterSave,
    };
    const next = await run(() => api.saveProvider(submission), t("providerForm.saved"));
    if (next) go("providers");
  };

  const canSwitchAfterSave =
    Boolean(form.apiKey?.trim()) ||
    Boolean(editingProvider && editingProvider.keyStatus !== "missing");

  return (
    <Page
      title={editingProvider ? t("providerForm.editTitle") : t("providerForm.addTitle")}
      back={() => go("providers")}
    >
      {providerId && !editingProvider ? (
        <EmptyState title={t("providerForm.missingTitle")} action={t("providerForm.missingAction")} />
      ) : (
        <form className="flex flex-col gap-3.5" onSubmit={submit}>
          <Card className="overflow-hidden">
            <div className="grid gap-4 p-4">
              <Field
                label={t("providerForm.name")}
                value={form.name}
                onValueChange={(name) => set({ name })}
                placeholder="DeepSeek"
              />
              <Field
                label="Base URL"
                value={form.endpoint}
                onValueChange={(endpoint) => set({ endpoint })}
                placeholder="https://api.deepseek.com/v1"
              />
              <Field
                label="API Key"
                value={form.apiKey ?? ""}
                onValueChange={(apiKey) => set({ apiKey })}
                placeholder="sk-..."
                autoComplete="off"
              />
              <ModelField
                label={t("providerForm.defaultModel")}
                value={form.model}
                options={modelOptions}
                loading={modelLoading}
                status={modelStatus}
                statusTone={modelStatusTone}
                onChange={(model) => set({ model })}
                onRefresh={
                  editingProvider && editingProvider.keyStatus !== "missing"
                    ? () => void loadModels(editingProvider)
                    : undefined
                }
              />
            </div>

            <button
              className="grid w-full grid-cols-[1fr_auto] items-center border-t border-stone-200 px-4 py-3 text-left transition hover:bg-stone-50"
              type="button"
              onClick={() => setAdvancedOpen((value) => !value)}
            >
              <span>
                <span className="block text-[15px] font-bold text-stone-950">{t("providerForm.advanced")}</span>
                <span className="mt-0.5 block text-[12px] text-stone-500">{t("providerForm.advancedDetail")}</span>
              </span>
              <ChevronDown
                className={advancedOpen ? "rotate-180 transition" : "transition"}
                size={21}
              />
            </button>

            {advancedOpen && (
              <div className="grid gap-4 border-t border-stone-100 p-4">
                <Field
                  label={t("providerForm.homepage")}
                  value={form.homepage ?? ""}
                  onValueChange={(homepage) => set({ homepage })}
                  placeholder="https://example.com"
                />
                <Field
                  label={t("providerForm.notes")}
                  value={form.notes ?? ""}
                  onValueChange={(notes) => set({ notes })}
                  placeholder={t("providerForm.notesPlaceholder")}
                  textarea
                />
                <div className="grid grid-cols-[1fr_auto] items-center gap-3 rounded-md border border-stone-200 bg-stone-50 px-3 py-3">
                  <span>
                    <span className="block text-[14px] font-bold text-stone-950">
                      {t("providerForm.disableImageGeneration")}
                    </span>
                    <span className="mt-0.5 block text-[12px] text-stone-500">
                      {t("providerForm.disableImageGenerationDetail")}
                    </span>
                  </span>
                  <Switch
                    checked={form.disableImageGeneration}
                    onChange={(event) =>
                      set({ disableImageGeneration: event.target.checked })
                    }
                  />
                </div>
              </div>
            )}
          </Card>

          <Button size="lg" tone="primary" disabled={busy}>
            {busy ? <Loader2 className="animate-spin" size={18} /> : <Save size={18} />}
            {t("providerForm.save")}
          </Button>

          <Card className="grid grid-cols-[1fr_auto] items-center gap-3 p-4">
            <span className="text-[14px] font-bold text-stone-950">
              {canSwitchAfterSave ? t("providerForm.setCurrent") : t("providerForm.setCurrentNeedsKey")}
            </span>
            <Switch
              checked={form.switchNow && canSwitchAfterSave}
              disabled={!canSwitchAfterSave}
              onChange={(event) => set({ switchNow: event.target.checked })}
            />
          </Card>
        </form>
      )}
    </Page>
  );
}
