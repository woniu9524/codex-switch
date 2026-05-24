import type { ReactNode } from "react";
import { Card } from "../../components/ui";

export function SettingSection({
  icon,
  title,
  children,
}: {
  icon: ReactNode;
  title: string;
  children: ReactNode;
}) {
  return (
    <Card className="overflow-hidden p-0">
      <div className="flex items-center gap-2 border-b border-stone-100 px-3 py-2">
        <div className="text-emerald-800">{icon}</div>
        <h2 className="text-[15px] font-black text-stone-950">{title}</h2>
      </div>
      <div className="divide-y divide-stone-100 px-3">{children}</div>
    </Card>
  );
}

export function SettingRow({
  label,
  detail,
  children,
}: {
  label: string;
  detail?: string;
  children: ReactNode;
}) {
  return (
    <div className="grid grid-cols-[1fr_auto] items-center gap-3 py-2">
      <div className="min-w-0">
        <div className="truncate text-[13px] font-semibold text-stone-950">{label}</div>
        {detail && <div className="mt-0.5 truncate text-[11px] text-stone-500">{detail}</div>}
      </div>
      <div className="flex min-w-0 items-center justify-end gap-2">{children}</div>
    </div>
  );
}
