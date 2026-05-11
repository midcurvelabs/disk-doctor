import { Check } from "lucide-react";
import { useApp } from "@/store";
import { formatBytes } from "@/lib/utils";

export function Done() {
  const result = useApp((s) => s.lastCleanResult);
  const scan = useApp((s) => s.scan);
  if (!result)
    return (
      <div className="flex h-full items-center justify-center text-muted">
        No result.
      </div>
    );
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8 text-center">
      <div className="flex size-12 items-center justify-center rounded-full bg-green/15 text-green">
        <Check size={24} />
      </div>
      <div className="font-mono text-[11px] tracking-[0.22em] uppercase text-green">
        Recovered
      </div>
      <div className="tabular text-5xl font-bold leading-none tracking-tight text-fg">
        {formatBytes(result.recovered_bytes)}
      </div>
      <div className="text-sm text-fg-2">
        Cleaned {result.per_category.length} categor
        {result.per_category.length === 1 ? "y" : "ies"}. History saved.
      </div>
      <button
        onClick={scan}
        className="mt-2 rounded-md border border-line bg-surface px-4 py-2 text-sm text-fg-2 hover:bg-line"
      >
        Re-scan
      </button>
    </div>
  );
}
