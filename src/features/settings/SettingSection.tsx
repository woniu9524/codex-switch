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
      <div className="flex items-center gap-3 px-4 py-3">
        <div className="text-emerald-800">{icon}</div>
        <h2 className="text-[17px] font-bold text-stone-950">{title}</h2>
      </div>
      <div className="divide-y divide-stone-100 px-4 pb-2">{children}</div>
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
    <div className="grid grid-cols-[1fr_auto] items-center gap-3 py-3">
      <div className="min-w-0">
        <div className="truncate text-[14px] font-medium text-stone-950">{label}</div>
        {detail && <div className="mt-0.5 truncate text-[12px] text-stone-500">{detail}</div>}
      </div>
      <div className="flex min-w-0 items-center justify-end gap-2">{children}</div>
    </div>
  );
}
