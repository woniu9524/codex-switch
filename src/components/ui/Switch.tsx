import type { InputHTMLAttributes } from "react";
import { cx } from "../../lib/ui";

export function Switch({
  className,
  ...props
}: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <label className={cx("relative inline-flex h-7 w-12 shrink-0 items-center", className)}>
      <input className="peer sr-only" type="checkbox" {...props} />
      <span className="absolute inset-0 rounded-full border border-stone-300 bg-stone-200 transition peer-checked:border-emerald-700 peer-checked:bg-emerald-700" />
      <span className="absolute left-1 size-5 rounded-full bg-white shadow-sm transition peer-checked:translate-x-5" />
    </label>
  );
}
