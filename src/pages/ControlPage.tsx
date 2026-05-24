import { Box, Copy, Globe2, Power, ShieldCheck, Terminal } from "lucide-react";
import type { RunAction, View } from "../app/types";
import { Button, Card } from "../components/ui";
import { ProviderAvatar } from "../features/providers/ProviderAvatar";
import { api } from "../lib/api";
import type { Snapshot } from "../lib/types";
import { cx } from "../lib/ui";
import { useI18n } from "../i18n";

export function ControlPage({
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
  const enabled = snapshot.state.enabled;
  const active = snapshot.activeProvider;
  const canEnable = Boolean(active);
  const { t } = useI18n();

  return (
    <div className="mx-auto flex w-full max-w-[560px] flex-col gap-3.5">
      <Card className="p-4">
        <div className="grid grid-cols-[1fr_auto] items-start gap-3">
          <div className="min-w-0">
            <div className="flex items-center gap-2.5">
              <span
                className={cx(
                  "status-dot size-3 rounded-full",
                  enabled ? "bg-emerald-600" : "bg-stone-300",
                )}
              />
              <h2 className="text-[18px] font-bold text-stone-950">
                {enabled ? t("control.proxyEnabled") : t("control.proxyDisabled")}
              </h2>
            </div>
            <p className="mt-2 truncate text-[14px] text-stone-500">
              {active ? `${active.name} · ${active.model}` : t("control.noProvider")}
            </p>
          </div>

          <Button
            className="min-w-[98px]"
            disabled={busy || (!enabled && !canEnable)}
            size="lg"
            tone={enabled ? "danger" : "primary"}
            onClick={() =>
              run(
                enabled ? api.disable : api.enable,
                enabled ? t("control.disabledSuccess") : t("control.enabledSuccess"),
              )
            }
          >
            {enabled ? <Power size={16} /> : <ShieldCheck size={16} />}
            {enabled ? t("control.disableProxy") : canEnable ? t("control.enableProxy") : t("control.addFirst")}
          </Button>
        </div>

        <div className="mt-4 divide-y divide-stone-100 border-t border-stone-100">
          <InfoRow icon={<Terminal size={18} />} label={t("control.port")} value={`127.0.0.1:${snapshot.state.proxyPort}`}>
            <button
              className="grid size-8 place-items-center rounded-md text-stone-500 transition hover:bg-stone-100 hover:text-stone-950"
              title={t("control.copyPort")}
              onClick={() => void navigator.clipboard?.writeText(`127.0.0.1:${snapshot.state.proxyPort}`)}
            >
              <Copy size={17} />
            </button>
          </InfoRow>
        </div>
      </Card>

      <Card className="p-4">
        <div className="mb-4 flex items-center justify-between gap-3">
          <h2 className="text-[18px] font-bold text-stone-950">{t("control.currentProvider")}</h2>
          <button
            className="inline-flex items-center gap-1 rounded-md px-2 py-1 text-[13px] font-medium text-emerald-800 transition hover:bg-emerald-50"
            onClick={() => go("providers")}
          >
            {t("control.changeProvider")}
            <span aria-hidden>›</span>
          </button>
        </div>

        {active ? (
          <div className="flex items-center gap-4">
            <ProviderAvatar provider={active} />
            <div className="min-w-0">
              <h3 className="truncate text-[18px] font-bold text-stone-950">{active.name}</h3>
              <div className="mt-2 flex min-w-0 items-center gap-2 text-[13px] text-stone-500">
                <Globe2 size={15} className="shrink-0" />
                <span className="truncate">{active.endpoint}</span>
              </div>
              <div className="mt-2 flex min-w-0 items-center gap-2 text-[13px] text-stone-500">
                <Box size={15} className="shrink-0" />
                <span className="truncate">{active.model}</span>
              </div>
            </div>
          </div>
        ) : (
          <div className="rounded-md bg-stone-50 p-4 text-[13px] leading-5 text-stone-500">
            {t("control.emptyProviderHint")}
          </div>
        )}
      </Card>

    </div>
  );
}

function InfoRow({
  icon,
  label,
  value,
  valueClass,
  children,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  valueClass?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className="grid grid-cols-[26px_64px_1fr_auto] items-center gap-2 py-3">
      <div className="text-stone-700">{icon}</div>
      <div className="text-[13px] text-stone-700">{label}</div>
      <div className={cx("truncate text-[14px] text-stone-950", valueClass)}>{value}</div>
      {children}
    </div>
  );
}
