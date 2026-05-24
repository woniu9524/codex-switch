import { ArrowLeft } from "lucide-react";
import type { ReactNode } from "react";
import { cx } from "../../lib/ui";
import { useI18n } from "../../i18n";

export function Page({
  title,
  actions,
  children,
  back,
  className,
  hideTitle = false,
  compact = false,
}: {
  title: string;
  actions?: ReactNode;
  children: ReactNode;
  back?: () => void;
  className?: string;
  hideTitle?: boolean;
  compact?: boolean;
}) {
  const { t } = useI18n();
  const showHeader = back || actions || !hideTitle;

  return (
    <div className={cx("mx-auto flex w-full max-w-[560px] flex-col", compact ? "gap-2.5" : "gap-3.5", className)}>
      {showHeader && (
        <div className={cx("flex items-center justify-between gap-3", compact ? "min-h-8" : "min-h-10")}>
          <div className="flex min-w-0 items-center gap-3">
            {back && (
              <button
                className={cx(
                  "grid shrink-0 place-items-center rounded-md text-stone-950 transition hover:bg-stone-100",
                  compact ? "size-8" : "size-9",
                )}
                onClick={back}
                title={t("nav.back")}
              >
                <ArrowLeft size={compact ? 20 : 23} strokeWidth={2} />
              </button>
            )}
            {!hideTitle && (
              <h1 className={cx("truncate font-black text-stone-950", compact ? "text-[20px]" : "text-[24px]")}>
                {title}
              </h1>
            )}
          </div>
          {actions && <div className="flex min-w-0 shrink-0 items-center gap-2">{actions}</div>}
        </div>
      )}
      {children}
    </div>
  );
}
