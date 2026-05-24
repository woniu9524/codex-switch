import { BarChart3, Building2, Settings, SlidersHorizontal } from "lucide-react";
import type { View } from "../../app/types";
import { cx } from "../../lib/ui";
import { useI18n, type MessageKey } from "../../i18n";

const items: Array<{
  labelKey: MessageKey;
  icon: typeof SlidersHorizontal;
  view: View;
  match: View[];
}> = [
  { labelKey: "nav.control", icon: SlidersHorizontal, view: "control", match: ["control"] },
  {
    labelKey: "nav.providers",
    icon: Building2,
    view: "providers",
    match: ["providers", "provider-form", "provider-import"],
  },
  { labelKey: "nav.stats", icon: BarChart3, view: "stats", match: ["stats", "stats-daily"] },
  { labelKey: "nav.settings", icon: Settings, view: "settings", match: ["settings"] },
];

export function BottomNav({
  view,
  go,
}: {
  view: View;
  go: (view: View) => void;
}) {
  const { t } = useI18n();

  return (
    <nav className="bottom-nav grid h-[62px] shrink-0 grid-cols-4 border-t border-stone-200 bg-white px-2 py-2">
      {items.map((item) => {
        const active = item.match.includes(view);
        const Icon = item.icon;
        return (
          <button
            key={item.labelKey}
            className={cx(
              "mx-auto flex h-full min-w-[72px] flex-col items-center justify-center gap-0.5 rounded-lg text-[12px] transition",
              active
                ? "bg-emerald-50 text-emerald-800"
                : "text-stone-500 hover:bg-stone-50 hover:text-stone-900",
            )}
            onClick={() => go(item.view)}
          >
            <Icon size={21} strokeWidth={2.2} />
            <span className="font-medium">{t(item.labelKey)}</span>
          </button>
        );
      })}
    </nav>
  );
}
