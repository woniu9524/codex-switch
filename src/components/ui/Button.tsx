import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cx } from "../../lib/ui";

type ButtonTone = "primary" | "secondary" | "ghost" | "danger";
type ButtonSize = "sm" | "md" | "lg";

const toneClasses: Record<ButtonTone, string> = {
  primary:
    "border-transparent bg-emerald-800 text-white shadow-[0_10px_22px_rgba(4,120,87,0.18)] hover:bg-emerald-900",
  secondary:
    "border-stone-300 bg-white text-stone-900 hover:border-emerald-700 hover:text-emerald-800",
  ghost: "border-transparent bg-transparent text-stone-600 hover:bg-stone-100 hover:text-stone-950",
  danger:
    "border-transparent bg-red-600 text-white shadow-[0_10px_22px_rgba(220,38,38,0.16)] hover:bg-red-700",
};

const sizeClasses: Record<ButtonSize, string> = {
  sm: "h-8 px-2.5 text-[12px]",
  md: "h-9 px-3 text-[13px]",
  lg: "h-10 px-4 text-[14px]",
};

export function Button({
  children,
  className,
  tone = "secondary",
  size = "md",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  children: ReactNode;
  tone?: ButtonTone;
  size?: ButtonSize;
}) {
  return (
    <button
      className={cx(
        "inline-flex shrink-0 items-center justify-center gap-1.5 rounded-md border font-semibold transition disabled:cursor-not-allowed disabled:opacity-45",
        toneClasses[tone],
        sizeClasses[size],
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
