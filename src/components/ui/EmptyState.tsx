export function EmptyState({ title, action }: { title: string; action: string }) {
  return (
    <div className="rounded-md border border-dashed border-stone-300 bg-white/70 px-4 py-8 text-center">
      <div className="text-[15px] font-bold text-stone-950">{title}</div>
      <div className="mt-1.5 text-[12px] text-stone-500">{action}</div>
    </div>
  );
}

export function EmptyInline({ children }: { children: React.ReactNode }) {
  return (
    <div className="rounded-md bg-stone-50 px-3 py-7 text-center text-[12px] font-medium text-stone-500">
      {children}
    </div>
  );
}
