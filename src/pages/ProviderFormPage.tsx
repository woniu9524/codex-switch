import React from "react";
import { ChevronDown, Loader2, Save } from "lucide-react";
import type { RunAction, View } from "../app/types";
import { Button, Card, EmptyState, Field, Switch } from "../components/ui";
import { Page } from "../components/layout/Page";
import { ModelField } from "../features/providers/ModelField";
import { newProviderForm, providerToForm } from "../features/providers/providerForm";
import { api } from "../lib/api";
import type { Provider, ProviderInput, Snapshot } from "../lib/types";

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

  React.useEffect(() => {
    setForm(initialForm);
    setModelOptions([]);
    setModelStatus(null);
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
        setModelStatus(`已拉取 ${result.models.length} 个模型`);
        return result.models;
      } catch (error) {
        if (!quiet) setModelStatus(`模型拉取失败，可手动输入：${String(error)}`);
        return null;
      } finally {
        setModelLoading(false);
      }
    },
    [applyModels],
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
    const next = await run(() => api.saveProvider(submission), "供应商已保存。");
    if (next) go("providers");
  };

  const canSwitchAfterSave =
    Boolean(form.apiKey?.trim()) ||
    Boolean(editingProvider && editingProvider.keyStatus !== "missing");

  return (
    <Page
      title={editingProvider ? "编辑供应商" : "添加供应商"}
      back={() => go("providers")}
    >
      {providerId && !editingProvider ? (
        <EmptyState title="供应商不存在" action="它可能已经被删除" />
      ) : (
        <form className="flex flex-col gap-3.5" onSubmit={submit}>
          <Card className="overflow-hidden">
            <div className="grid gap-4 p-4">
              <Field
                label="名称"
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
                label="默认模型"
                value={form.model}
                options={modelOptions}
                loading={modelLoading}
                status={modelStatus}
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
                <span className="block text-[15px] font-bold text-stone-950">高级信息</span>
                <span className="mt-0.5 block text-[12px] text-stone-500">官网、备注</span>
              </span>
              <ChevronDown
                className={advancedOpen ? "rotate-180 transition" : "transition"}
                size={21}
              />
            </button>

            {advancedOpen && (
              <div className="grid gap-4 border-t border-stone-100 p-4">
                <Field
                  label="官网（可选）"
                  value={form.homepage ?? ""}
                  onValueChange={(homepage) => set({ homepage })}
                  placeholder="https://example.com"
                />
                <Field
                  label="备注（可选）"
                  value={form.notes ?? ""}
                  onValueChange={(notes) => set({ notes })}
                  placeholder="计费说明、模型列表等"
                  textarea
                />
                <div className="grid grid-cols-[1fr_auto] items-center gap-3 rounded-md border border-stone-200 bg-stone-50 px-3 py-3">
                  <span>
                    <span className="block text-[14px] font-bold text-stone-950">
                      关闭图片生成工具
                    </span>
                    <span className="mt-0.5 block text-[12px] text-stone-500">
                      发送请求前移除 image_generation 工具
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
            保存
          </Button>

          <Card className="grid grid-cols-[1fr_auto] items-center gap-3 p-4">
            <span className="text-[14px] font-bold text-stone-950">
              {canSwitchAfterSave ? "设为当前供应商" : "填写 API Key 后可设为当前"}
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
