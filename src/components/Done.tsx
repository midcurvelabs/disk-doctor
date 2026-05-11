import { Check } from "lucide-react";
import { useApp } from "@/store";
import { formatBytes } from "@/lib/utils";

export function Done() {
  const result = useApp((s) => s.lastCleanResult);
  const scan = useApp((s) => s.scan);
  if (!result)
    return (
      <div className="flex h-full items-center justify-center text-fg-2">
        No result.
      </div>
    );
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 px-8 text-center">
      <div className="flex size-9 items-center justify-center rounded-full bg-green/15 text-green">
        <Check size={18} strokeWidth={2.5} />
      </div>
      <div className="text-[11px] uppercase tracking-wide text-fg-3">
        Recovered
      </div>
      <div className="tabular text-[40px] font-semibold leading-none tracking-tight text-fg">
        {formatBytes(result.recovered_bytes)}
      </div>
      <div className="text-[12px] text-fg-2">
        Cleaned {result.per_category.length} categor
        {result.per_category.length === 1 ? "y" : "ies"}.
      </div>
      <button
        onClick={scan}
        className="mt-1 rounded-md bg-surface px-3 py-1.5 text-[12px] text-fg-2 hover:bg-surface-2"
      >
        Re-scan
      </button>
    </div>
  );
}
