import React from "react";
import { ChevronDown, Download, Loader2 } from "lucide-react";
import { commonModelOptions } from "./constants";
import { cx } from "../../lib/ui";
import { useI18n } from "../../i18n";

export type ModelStatusTone = "success" | "warning";

export function ModelField({
  label,
  value,
  options,
  loading,
  status,
  statusTone = "success",
  onChange,
  onRefresh,
}: {
  label: string;
  value: string;
  options: string[];
  loading: boolean;
  status: string | null;
  statusTone?: ModelStatusTone;
  onChange: (value: string) => void;
  onRefresh?: () => void;
}) {
  const [custom, setCustom] = React.useState(false);
  const { t } = useI18n();
  const baseChoices = React.useMemo(
    () => Array.from(new Set([...options, ...commonModelOptions].filter(Boolean))),
    [options],
  );
  const choices = baseChoices.includes(value) || !value ? baseChoices : [value, ...baseChoices];
  const selectValue = custom ? "__custom__" : choices.includes(value) ? value : choices[0] ?? "";
  const fieldClass =
    "w-full rounded-md border border-stone-300 bg-white px-3 text-[14px] text-stone-950 outline-none transition placeholder:text-stone-400 focus:border-emerald-600 focus:ring-3 focus:ring-emerald-100";

  React.useEffect(() => {
    if (baseChoices.includes(value)) setCustom(false);
  }, [baseChoices, value]);

  return (
    <label className="flex min-w-0 flex-col gap-2">
      <span className="flex items-center justify-between gap-3">
        <span className="text-[14px] font-bold text-stone-950">{label}</span>
        {onRefresh && (
          <button
            type="button"
            className="inline-flex h-7 items-center gap-1 rounded-md px-2 text-[12px] font-bold text-emerald-800 transition hover:bg-emerald-50 disabled:opacity-50"
            disabled={loading}
            onClick={(event) => {
              event.preventDefault();
              onRefresh();
            }}
          >
            {loading ? <Loader2 className="animate-spin" size={13} /> : <Download size={13} />}
            {t("providerForm.fetchModels")}
          </button>
        )}
      </span>
      <span className="relative block">
        <select
          className={cx(fieldClass, "h-10 cursor-pointer appearance-none pr-10")}
          value={selectValue}
          onChange={(event) => {
            if (event.target.value === "__custom__") {
              setCustom(true);
              return;
            }
            setCustom(false);
            onChange(event.target.value);
          }}
        >
          {choices.map((model) => (
            <option value={model} key={model}>
              {model}
            </option>
          ))}
          <option value="__custom__">{t("providerForm.customModel")}</option>
        </select>
        <ChevronDown
          className="pointer-events-none absolute right-4 top-1/2 -translate-y-1/2 text-stone-500"
          size={18}
        />
      </span>
      {custom && (
        <input
          className={cx(fieldClass, "h-10")}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          placeholder={loading ? t("providerForm.loadingModels") : t("providerForm.modelPlaceholder")}
        />
      )}
      {status && (
        <span
          className={cx(
            "truncate text-[12px] font-semibold",
            statusTone === "warning" ? "text-amber-700" : "text-emerald-800",
          )}
        >
          {status}
        </span>
      )}
    </label>
  );
}
