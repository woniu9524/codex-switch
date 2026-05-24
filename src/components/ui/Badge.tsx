import type { ReactNode } from "react";
import { cx } from "../../lib/ui";

type BadgeTone = "active" | "warn" | "neutral" | "danger";

export function Badge({
  children,
  tone = "neutral",
}: {
  children: ReactNode;
  tone?: BadgeTone;
}) {
  return (
    <span
      className={cx(
        "inline-flex h-5 shrink-0 items-center rounded border px-1.5 text-[11px] font-semibold",
        tone === "active" && "border-emerald-200 bg-emerald-50 text-emerald-800",
        tone === "warn" && "border-amber-200 bg-amber-50 text-amber-700",
        tone === "danger" && "border-red-200 bg-red-50 text-red-700",
        tone === "neutral" && "border-stone-200 bg-stone-50 text-stone-600",
      )}
    >
      {children}
    </span>
  );
}
