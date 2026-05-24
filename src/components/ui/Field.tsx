import type { InputHTMLAttributes, TextareaHTMLAttributes } from "react";
import { cx } from "../../lib/ui";

const fieldClass =
  "w-full rounded-md border border-stone-300 bg-white px-3 text-[14px] text-stone-950 outline-none transition placeholder:text-stone-400 focus:border-emerald-600 focus:ring-3 focus:ring-emerald-100";

export function Field({
  label,
  value,
  onValueChange,
  placeholder,
  type = "text",
  textarea = false,
  className,
  autoComplete,
}: {
  label: string;
  value: string;
  onValueChange: (value: string) => void;
  placeholder?: string;
  type?: string;
  textarea?: boolean;
  className?: string;
  autoComplete?: string;
}) {
  return (
    <label className={cx("flex min-w-0 flex-col gap-2", className)}>
      <span className="text-[14px] font-bold text-stone-950">{label}</span>
      {textarea ? (
        <textarea
          className={cx(fieldClass, "min-h-28 resize-y py-3 leading-6")}
          value={value}
          onChange={(event) => onValueChange(event.target.value)}
          placeholder={placeholder}
        />
      ) : (
        <input
          className={cx(fieldClass, "h-10")}
          type={type}
          value={value}
          onChange={(event) => onValueChange(event.target.value)}
          placeholder={placeholder}
          autoComplete={autoComplete}
        />
      )}
    </label>
  );
}

export function TextInput(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input className={cx(fieldClass, "h-9 text-[13px]", props.className)} {...props} />;
}

export function TextArea(props: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      className={cx(fieldClass, "min-h-28 resize-y py-3 text-[13px] leading-6", props.className)}
      {...props}
    />
  );
}

export function PasswordField({
  label,
  value,
  onValueChange,
  placeholder,
}: {
  label: string;
  value: string;
  onValueChange: (value: string) => void;
  placeholder?: string;
}) {
  return (
    <label className="flex min-w-0 flex-col gap-2">
      <span className="text-[14px] font-bold text-stone-950">{label}</span>
      <span className="relative block">
        <input
          className={cx(fieldClass, "h-10 pr-3")}
          type="password"
          value={value}
          onChange={(event) => onValueChange(event.target.value)}
          placeholder={placeholder}
          autoComplete="new-password"
        />
      </span>
    </label>
  );
}
