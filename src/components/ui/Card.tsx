import type { HTMLAttributes, ReactNode } from "react";
import { cx } from "../../lib/ui";

export function Card({
  children,
  className,
  ...props
}: HTMLAttributes<HTMLElement> & { children: ReactNode }) {
  return (
    <section
      className={cx("rounded-md border border-stone-200 bg-white shadow-sm", className)}
      {...props}
    >
      {children}
    </section>
  );
}
