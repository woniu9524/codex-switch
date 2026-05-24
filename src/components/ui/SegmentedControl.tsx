import { cx } from "../../lib/ui";

export function SegmentedControl<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: Array<{ label: string; value: T }>;
  onChange: (value: T) => void;
}) {
  return (
    <div className="grid h-9 overflow-hidden rounded-md border border-stone-300 bg-white" style={{ gridTemplateColumns: `repeat(${options.length}, minmax(0, 1fr))` }}>
      {options.map((option) => (
        <button
          key={option.value}
          className={cx(
            "border-l border-stone-200 px-3 text-[13px] font-semibold first:border-l-0",
            option.value === value
              ? "bg-emerald-50 text-emerald-800 ring-1 ring-inset ring-emerald-500"
              : "text-stone-800 hover:bg-stone-50",
          )}
          type="button"
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
