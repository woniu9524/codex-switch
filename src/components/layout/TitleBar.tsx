import { Minus, Square, X } from "lucide-react";
import type { ThemeMode } from "../../lib/types";
import { controlWindow } from "../../lib/window";
import { cx } from "../../lib/ui";
import { useI18n } from "../../i18n";

export function TitleBar({
  online,
}: {
  online: boolean;
  themeMode: ThemeMode;
}) {
  const { t } = useI18n();

  return (
    <header
      className="title-bar flex h-[46px] shrink-0 select-none items-center border-b border-stone-200 bg-white/95 px-4 text-stone-950"
      data-tauri-drag-region
    >
      <button
        className="inline-flex items-center gap-2.5 text-[17px] font-bold"
        title="codex-switch"
      >
        <span>codex-switch</span>
        <span
          className={cx(
            "status-dot size-2.5 rounded-full shadow-[0_0_0_3px_rgba(22,163,74,0.08)]",
            online ? "bg-emerald-600" : "bg-stone-300",
          )}
        />
      </button>

      <div className="ml-auto flex items-center gap-1.5">
        <WindowButton label={t("window.minimize")} onClick={() => void controlWindow("minimize")}>
          <Minus size={18} strokeWidth={2} />
        </WindowButton>
        <WindowButton label={t("window.maximize")} onClick={() => void controlWindow("maximize")}>
          <Square size={16} strokeWidth={2} />
        </WindowButton>
        <WindowButton danger label={t("window.closeToTray")} onClick={() => void controlWindow("close")}>
          <X size={20} strokeWidth={2} />
        </WindowButton>
      </div>
    </header>
  );
}

function WindowButton({
  children,
  label,
  danger = false,
  onClick,
}: {
  children: React.ReactNode;
  label: string;
  danger?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      className={cx(
        "window-button grid size-8 place-items-center rounded-md transition",
        danger
          ? "text-stone-950 hover:bg-red-500 hover:text-white"
          : "text-stone-950 hover:bg-stone-100",
      )}
      title={label}
      onClick={onClick}
    >
      {children}
    </button>
  );
}
