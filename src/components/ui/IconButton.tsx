import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cx } from "../../lib/ui";

export function IconButton({
  children,
  className,
  danger = false,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  children: ReactNode;
  danger?: boolean;
}) {
  return (
    <button
      className={cx(
        "grid size-8 shrink-0 place-items-center rounded-md border bg-white text-stone-600 transition hover:bg-stone-100 hover:text-stone-950 disabled:cursor-not-allowed disabled:opacity-45",
        danger ? "border-red-200 text-red-600 hover:bg-red-50" : "border-stone-300",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
