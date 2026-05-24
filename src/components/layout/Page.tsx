import { ArrowLeft } from "lucide-react";
import type { ReactNode } from "react";
import { cx } from "../../lib/ui";

export function Page({
  title,
  actions,
  children,
  back,
  className,
}: {
  title: string;
  actions?: ReactNode;
  children: ReactNode;
  back?: () => void;
  className?: string;
}) {
  return (
    <div className={cx("mx-auto flex w-full max-w-[560px] flex-col gap-3.5", className)}>
      <div className="flex min-h-10 items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-3">
          {back && (
            <button
              className="grid size-9 shrink-0 place-items-center rounded-md text-stone-950 transition hover:bg-stone-100"
              onClick={back}
              title="返回"
            >
              <ArrowLeft size={23} strokeWidth={2} />
            </button>
          )}
          <h1 className="truncate text-[24px] font-black text-stone-950">
            {title}
          </h1>
        </div>
        {actions && <div className="flex shrink-0 items-center gap-3">{actions}</div>}
      </div>
      {children}
    </div>
  );
}
