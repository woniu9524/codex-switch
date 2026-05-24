import React from "react";
import { Activity, ArrowRight, BarChart3, Gauge, Loader2, PieChart, Sigma } from "lucide-react";
import type { View } from "../app/types";
import { Page } from "../components/layout/Page";
import { Card } from "../components/ui";
import { api } from "../lib/api";
import type { DailyTokenUsage, ModelTokenUsage, StatsSummary, TokenUsage } from "../lib/types";
import { cx } from "../lib/ui";
import { compactNumber, formatDecimal } from "../lib/format";
import { useI18n } from "../i18n";

const MODEL_LIMIT = 5;
const RECENT_DAYS = 7;

export function StatsPage({ go }: { go: (view: View) => void }) {
  return <StatsShell render={(stats) => <StatsOverview stats={stats} go={go} />} />;
}

export function StatsDailyPage({ go }: { go: (view: View) => void }) {
  const { t } = useI18n();
  return (
    <StatsShell
      fallbackPage={{ title: t("stats.dailyFullTitle"), back: () => go("stats"), compact: true }}
      render={(stats) => (
        <Page title={t("stats.dailyFullTitle")} back={() => go("stats")} compact>
          <DailyUsageTable days={[...stats.dailyUsage].reverse()} full />
        </Page>
      )}
    />
  );
}

function StatsShell({
  render,
  fallbackPage,
}: {
  render: (stats: StatsSummary) => React.ReactNode;
  fallbackPage?: {
    title: string;
    back?: () => void;
    compact?: boolean;
    hideTitle?: boolean;
  };
}) {
  const { t } = useI18n();
  const page = fallbackPage ?? { title: t("stats.title"), hideTitle: true, compact: true };
  const [stats, setStats] = React.useState<StatsSummary | null>(null);
  const [error, setError] = React.useState<string | null>(null);
  const [loading, setLoading] = React.useState(true);

  React.useEffect(() => {
    let cancelled = false;

    setLoading(true);
    setError(null);
    void api
      .statsSummary()
      .then((summary) => {
        if (!cancelled) setStats(summary);
      })
      .catch((reason) => {
        if (!cancelled) setError(String(reason));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  if (loading) {
    return (
      <Page {...page}>
        <div className="grid min-h-[220px] place-items-center text-stone-400">
          <Loader2 className="animate-spin" size={24} />
        </div>
      </Page>
    );
  }

  if (error) {
    return (
      <Page {...page}>
        <Card className="p-3 text-[13px] leading-5 text-red-700">{error}</Card>
      </Page>
    );
  }

  if (!stats || stats.usageEventCount === 0) {
    return (
      <Page {...page}>
        <Card className="p-4">
          <div className="flex items-start gap-3">
            <div className="grid size-9 shrink-0 place-items-center rounded-md bg-stone-100 text-stone-700">
              <BarChart3 size={19} />
            </div>
            <div>
              <h2 className="text-[16px] font-bold text-stone-950">{t("stats.emptyTitle")}</h2>
              <p className="mt-1 text-[13px] leading-5 text-stone-500">{t("stats.emptyDetail")}</p>
            </div>
          </div>
        </Card>
      </Page>
    );
  }

  return <>{render(stats)}</>;
}

function StatsOverview({ stats, go }: { stats: StatsSummary; go: (view: View) => void }) {
  const { locale, t } = useI18n();
  const recentDays = stats.dailyUsage.slice(-RECENT_DAYS).reverse();

  return (
    <Page title={t("stats.title")} hideTitle compact>
      <div className="grid grid-cols-4 overflow-hidden rounded-md border border-stone-200 bg-white shadow-sm">
        <MetricCell icon={<Sigma size={14} />} label={t("stats.allTime")} value={compactNumber(stats.allTimeUsage.totalTokens, locale)} />
        <MetricCell icon={<Activity size={14} />} label={t("stats.today")} value={compactNumber(stats.todayUsage.totalTokens, locale)} />
        <MetricCell icon={<BarChart3 size={14} />} label={t("stats.last7Days")} value={compactNumber(stats.last7DaysUsage.totalTokens, locale)} />
        <MetricCell
          icon={<Gauge size={14} />}
          label={t("stats.outputSpeed")}
          value={formatDecimal(stats.speed.outputTokensPerSecond, 1)}
          suffix="tok/s"
        />
      </div>

      <DailyUsageTable
        days={recentDays}
        action={
          <button
            className="inline-flex items-center gap-1 rounded px-1.5 py-1 text-[12px] font-bold text-emerald-800 transition hover:bg-emerald-50"
            onClick={() => go("stats-daily")}
          >
            {t("stats.moreDaily")}
            <ArrowRight size={13} />
          </button>
        }
      />

      <UsageBreakdown usage={stats.last7DaysUsage} />
      <ModelUsageList models={stats.modelUsage.slice(0, MODEL_LIMIT)} />
    </Page>
  );
}

function MetricCell({
  icon,
  label,
  value,
  suffix,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  suffix?: string;
}) {
  return (
    <div className="min-w-0 border-l border-stone-100 px-2.5 py-2 first:border-l-0">
      <div className="mb-1 flex min-w-0 items-center gap-1 text-stone-500">
        <span className="shrink-0 text-emerald-800">{icon}</span>
        <span className="truncate text-[11px] font-semibold">{label}</span>
      </div>
      <div className="flex min-w-0 items-baseline gap-1">
        <div className="truncate text-[18px] font-black leading-none text-stone-950">{value}</div>
        {suffix && <div className="shrink-0 text-[10px] font-bold text-stone-500">{suffix}</div>}
      </div>
    </div>
  );
}

function DailyUsageTable({
  days,
  action,
  full = false,
}: {
  days: DailyTokenUsage[];
  action?: React.ReactNode;
  full?: boolean;
}) {
  const { t } = useI18n();

  return (
    <Card className="overflow-hidden">
      <div className="flex items-center justify-between gap-2 border-b border-stone-100 px-3 py-2">
        <div className="flex min-w-0 items-baseline gap-2">
          <h2 className="truncate text-[15px] font-black text-stone-950">{t("stats.daily")}</h2>
          <span className="shrink-0 text-[11px] font-semibold text-stone-500">
            {full ? t("stats.last30Days") : t("stats.last7Days")}
          </span>
        </div>
        {action}
      </div>
      <div className="grid grid-cols-[42px_1fr_repeat(4,minmax(36px,0.62fr))] gap-2 border-b border-stone-100 px-3 py-1.5 text-right text-[11px] font-bold text-stone-500">
        <span className="min-w-0 truncate text-left">{t("stats.date")}</span>
        <span className="min-w-0 truncate">{t("stats.total")}</span>
        <span className="min-w-0 truncate">{t("stats.input")}</span>
        <span className="min-w-0 truncate">{t("stats.output")}</span>
        <span className="min-w-0 truncate">{t("stats.cached")}</span>
        <span className="min-w-0 truncate">{t("stats.reasoning")}</span>
      </div>
      <div className={cx("divide-y divide-stone-100", full && "max-h-none")}>
        {days.map((item) => (
          <DailyUsageRow key={item.date} item={item} />
        ))}
      </div>
    </Card>
  );
}

function DailyUsageRow({ item }: { item: DailyTokenUsage }) {
  const { locale } = useI18n();
  const usage = item.usage;

  return (
    <div className="grid grid-cols-[42px_1fr_repeat(4,minmax(36px,0.62fr))] items-center gap-2 px-3 py-1.5 text-right text-[12px]">
      <div className="truncate text-left font-semibold text-stone-600">{item.date.slice(5)}</div>
      <StatNumber value={usage.totalTokens} strong locale={locale} />
      <StatNumber value={usage.inputTokens} locale={locale} />
      <StatNumber value={usage.outputTokens} locale={locale} />
      <StatNumber value={usage.cachedInputTokens} locale={locale} />
      <StatNumber value={usage.reasoningOutputTokens} locale={locale} />
    </div>
  );
}

function StatNumber({ value, locale, strong = false }: { value: number; locale: "zh-CN" | "en-US"; strong?: boolean }) {
  return (
    <span
      className={cx(
        "truncate",
        strong ? "font-black text-stone-950" : "font-semibold",
        value === 0 && !strong ? "text-stone-300" : "text-stone-700",
      )}
    >
      {compactNumber(value, locale)}
    </span>
  );
}

function UsageBreakdown({ usage }: { usage: TokenUsage }) {
  const { locale, t } = useI18n();
  const rows = [
    { label: t("stats.input"), value: usage.inputTokens, className: "bg-emerald-700" },
    { label: t("stats.output"), value: usage.outputTokens, className: "bg-teal-500" },
    { label: t("stats.cached"), value: usage.cachedInputTokens, className: "bg-amber-500" },
    { label: t("stats.reasoning"), value: usage.reasoningOutputTokens, className: "bg-stone-600" },
  ];
  const max = Math.max(...rows.map((row) => row.value), 1);

  return (
    <Card className="p-3">
      <CompactSectionTitle icon={<PieChart size={15} />} title={t("stats.breakdown")} />
      <div className="mt-2 grid grid-cols-2 gap-x-4 gap-y-2">
        {rows.map((row) => (
          <div key={row.label} className="min-w-0">
            <div className="mb-1 flex items-center justify-between gap-2">
              <div className="truncate text-[12px] font-semibold text-stone-600">{row.label}</div>
              <div className="shrink-0 text-[12px] font-black text-stone-950">{compactNumber(row.value, locale)}</div>
            </div>
            <div className="h-1.5 overflow-hidden rounded-full bg-stone-100">
              <div
                className={cx("h-full rounded-full", row.className)}
                style={{ width: `${row.value === 0 ? 0 : Math.max(6, (row.value / max) * 100)}%` }}
              />
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
}

function ModelUsageList({ models }: { models: ModelTokenUsage[] }) {
  const { locale, t } = useI18n();
  const max = Math.max(...models.map((item) => item.usage.totalTokens), 1);

  return (
    <Card className="p-3">
      <CompactSectionTitle icon={<Gauge size={15} />} title={t("stats.models")} />
      <div className="mt-2 space-y-2">
        {models.length === 0 ? (
          <div className="rounded-md bg-stone-50 px-3 py-2 text-[12px] text-stone-500">{t("stats.noModels")}</div>
        ) : (
          models.map((item) => (
            <div key={`${item.providerId}/${item.model}`} className="grid grid-cols-[1fr_62px] items-center gap-3">
              <div className="min-w-0">
                <div className="truncate text-[12px] font-bold text-stone-950">
                  {item.providerId} / {item.model}
                </div>
                <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-stone-100">
                  <div
                    className="h-full rounded-full bg-teal-500"
                    style={{ width: `${Math.max(5, (item.usage.totalTokens / max) * 100)}%` }}
                  />
                </div>
              </div>
              <div className="text-right text-[12px] font-black text-stone-950">
                {compactNumber(item.usage.totalTokens, locale)}
              </div>
            </div>
          ))
        )}
      </div>
    </Card>
  );
}

function CompactSectionTitle({ icon, title }: { icon: React.ReactNode; title: string }) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="text-emerald-800">{icon}</span>
      <h2 className="text-[15px] font-black text-stone-950">{title}</h2>
    </div>
  );
}
