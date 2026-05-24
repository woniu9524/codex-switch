import { Building2, Settings, SlidersHorizontal } from "lucide-react";
import type { View } from "../../app/types";
import { cx } from "../../lib/ui";

const items: Array<{
  label: string;
  icon: typeof SlidersHorizontal;
  view: View;
  match: View[];
}> = [
  { label: "控制", icon: SlidersHorizontal, view: "control", match: ["control"] },
  {
    label: "供应商",
    icon: Building2,
    view: "providers",
    match: ["providers", "provider-form", "provider-import"],
  },
  { label: "设置", icon: Settings, view: "settings", match: ["settings"] },
];

export function BottomNav({
  view,
  go,
}: {
  view: View;
  go: (view: View) => void;
}) {
  return (
    <nav className="bottom-nav grid h-[62px] shrink-0 grid-cols-3 border-t border-stone-200 bg-white px-3 py-2">
      {items.map((item) => {
        const active = item.match.includes(view);
        const Icon = item.icon;
        return (
          <button
            key={item.label}
            className={cx(
              "mx-auto flex h-full min-w-[88px] flex-col items-center justify-center gap-0.5 rounded-lg text-[12px] transition",
              active
                ? "bg-emerald-50 text-emerald-800"
                : "text-stone-500 hover:bg-stone-50 hover:text-stone-900",
            )}
            onClick={() => go(item.view)}
          >
            <Icon size={21} strokeWidth={2.2} />
            <span className="font-medium">{item.label}</span>
          </button>
        );
      })}
    </nav>
  );
}
