import React from "react";
import { Link, Loader2, Upload } from "lucide-react";
import type { RunAction, View } from "../app/types";
import { Button, Card, EmptyInline, Switch, TextArea } from "../components/ui";
import { Page } from "../components/layout/Page";
import { api } from "../lib/api";
import type { ImportPreview, Snapshot } from "../lib/types";
import { useI18n } from "../i18n";

export function ImportProviderPage({
  snapshot,
  busy,
  run,
  go,
}: {
  snapshot: Snapshot;
  busy: boolean;
  run: RunAction;
  go: (view: View) => void;
}) {
  const [url, setUrl] = React.useState("");
  const [preview, setPreview] = React.useState<ImportPreview | null>(null);
  const [parseError, setParseError] = React.useState<string | null>(null);
  const [switchNow, setSwitchNow] = React.useState(false);
  const { t } = useI18n();

  const parse = async () => {
    setParseError(null);
    setPreview(null);
    try {
      const next = await api.parseImportUrl(url);
      setPreview(next);
      setSwitchNow((next.enabledHint || !snapshot.state.activeProviderId) && next.hasApiKey);
    } catch (error) {
      setParseError(String(error));
    }
  };

  const confirm = async () => {
    const effectiveSwitchNow = switchNow && Boolean(preview?.hasApiKey);
    const next = await run(
      () => api.importProvider(url, effectiveSwitchNow),
      effectiveSwitchNow ? t("import.importedAndSwitched") : t("import.imported"),
    );
    if (next) go("providers");
  };

  return (
    <Page title={t("import.title")} back={() => go("providers")}>
      <Card className="p-4">
        <div className="mb-3 text-[13px] font-medium text-stone-700">
          {t("import.pasteHint")}
        </div>
        <TextArea
          value={url}
          onChange={(event) => setUrl(event.target.value)}
          placeholder="codexswitch://v1/import?resource=provider&name=DeepSeek&endpoint=..."
        />
        <Button className="mt-5" disabled={!url.trim()} onClick={parse}>
          <Link size={16} />
          {t("import.parse")}
        </Button>
        {parseError && (
          <div className="mt-3 rounded-md bg-red-50 px-3 py-2 text-[12px] font-semibold text-red-700">
            {parseError}
          </div>
        )}
      </Card>

      <Card className="overflow-hidden">
        <div className="border-b border-stone-100 px-4 py-3 text-[17px] font-bold text-stone-950">
          {t("import.preview")}
        </div>
        {!preview ? (
          <div className="p-4">
            <EmptyInline>{t("import.previewEmpty")}</EmptyInline>
          </div>
        ) : (
          <div className="divide-y divide-stone-100 px-4">
            <PreviewRow
              label={t("import.source")}
              value={preview.scheme === "ccswitch" ? t("import.ccSwitchCompatible") : "codex-switch"}
            />
            <PreviewRow label={t("import.name")} value={preview.name} />
            <PreviewRow label="Endpoint" value={preview.endpoint} />
            <PreviewRow label={t("import.model")} value={preview.model} />
            <PreviewRow label="Key" value={preview.apiKeyMasked ?? t("import.keyMissing")} accent={preview.hasApiKey} />
          </div>
        )}
      </Card>

      {preview && (
        <>
          <Card className="grid grid-cols-[1fr_auto] items-center gap-3 p-4">
            <span className="text-[14px] font-bold text-stone-950">
              {preview.hasApiKey ? t("import.setCurrent") : t("import.setCurrentNeedsKey")}
            </span>
            <Switch
              checked={switchNow && preview.hasApiKey}
              disabled={!preview.hasApiKey}
              onChange={(event) => setSwitchNow(event.target.checked)}
            />
          </Card>
          <Button size="lg" tone="primary" disabled={busy} onClick={confirm}>
            {busy ? <Loader2 className="animate-spin" size={18} /> : <Upload size={18} />}
            {t("import.confirm")}
          </Button>
        </>
      )}
    </Page>
  );
}

function PreviewRow({
  label,
  value,
  accent = false,
}: {
  label: string;
  value: string;
  accent?: boolean;
}) {
  return (
    <div className="grid grid-cols-[86px_1fr] gap-3 py-3 text-[13px]">
      <span className="font-medium text-stone-600">{label}</span>
      <span className={accent ? "break-words text-right font-bold text-emerald-800" : "break-words text-right text-stone-800"}>
        {value}
      </span>
    </div>
  );
}
