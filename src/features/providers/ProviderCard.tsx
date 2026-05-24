import { useEffect, useRef, useState } from "react";
import { Box, Edit3, Globe2, MoreVertical, Trash2 } from "lucide-react";
import { Badge, Button, IconButton } from "../../components/ui";
import type { Provider } from "../../lib/types";
import { cx } from "../../lib/ui";
import { ProviderAvatar } from "./ProviderAvatar";

export function ProviderCard({
  provider,
  isCurrent,
  busy,
  onSwitch,
  onEdit,
  onDelete,
}: {
  provider: Provider;
  isCurrent: boolean;
  busy: boolean;
  onSwitch: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const missingKey = provider.keyStatus === "missing";
  const canDelete = !isCurrent;
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menuOpen) return;

    const handlePointerDown = (event: PointerEvent) => {
      if (!menuRef.current?.contains(event.target as Node)) {
        setMenuOpen(false);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setMenuOpen(false);
      }
    };

    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [menuOpen]);

  const handleDelete = () => {
    setMenuOpen(false);
    onDelete();
  };

  return (
    <article
      className={cx(
        "grid grid-cols-[14px_52px_1fr_auto] items-center gap-2.5 rounded-md border bg-white p-3 transition",
        isCurrent
          ? "border-emerald-600 shadow-[inset_3px_0_0_#059669]"
          : "border-stone-200 hover:border-stone-300",
      )}
    >
      <span
        className={cx(
          "status-dot size-3.5 rounded-full border-2",
          isCurrent && "border-emerald-700 bg-emerald-600",
          !isCurrent && missingKey && "border-amber-400 bg-amber-400",
          !isCurrent && !missingKey && "border-stone-500 bg-white",
        )}
      />
      <ProviderAvatar provider={provider} />

      <div className="min-w-0">
        <div className="flex min-w-0 items-center gap-2">
          <h2 className="truncate text-[16px] font-bold text-stone-950">{provider.name}</h2>
          {isCurrent && <Badge tone="active">使用中</Badge>}
          {missingKey && <Badge tone="warn">缺 Key</Badge>}
          {provider.disableImageGeneration && <Badge>无图片工具</Badge>}
        </div>
        <div className="mt-1 flex min-w-0 items-center gap-1.5 text-[12px] text-stone-500">
          <Box size={14} className="shrink-0" />
          <span className="truncate">{provider.model}</span>
        </div>
        <div className="mt-1 flex min-w-0 items-center gap-1.5 text-[12px] text-stone-500">
          <Globe2 size={14} className="shrink-0" />
          <span className="truncate">{provider.endpoint}</span>
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-1.5">
        {!isCurrent && (
          <Button size="sm" disabled={busy || missingKey} onClick={onSwitch}>
            切换
          </Button>
        )}
        <Button size="sm" onClick={onEdit}>
          <Edit3 size={14} />
          编辑
        </Button>
        {canDelete && (
          <div ref={menuRef} className="relative">
            <IconButton
              title="更多"
              aria-haspopup="menu"
              aria-expanded={menuOpen}
              onClick={() => setMenuOpen((open) => !open)}
            >
              <MoreVertical size={18} />
            </IconButton>
            {menuOpen && (
              <div
                role="menu"
                className="absolute right-0 top-9 z-20 min-w-[104px] rounded-md border border-stone-200 bg-white p-1 shadow-lg"
              >
                <button
                  type="button"
                  role="menuitem"
                  className="flex w-full items-center gap-2 rounded px-2.5 py-2 text-left text-[13px] font-medium text-red-600 transition hover:bg-red-50"
                  onClick={handleDelete}
                >
                  <Trash2 size={14} />
                  删除
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </article>
  );
}
