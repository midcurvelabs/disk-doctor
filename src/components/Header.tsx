import { RotateCw } from "lucide-react";
import { useApp } from "@/store";
import { formatBytes, formatPercent } from "@/lib/utils";

export function Header() {
  const disk = useApp((s) => s.disk);
  const mem = useApp((s) => s.mem);
  const scan = useApp((s) => s.scan);
  const refresh = useApp((s) => s.refreshStatus);

  const pct = disk?.free_percent ?? 0;
  const pctColor =
    pct < 10 ? "text-red" : pct < 25 ? "text-accent-2" : "text-green";
  const memColor =
    mem?.state === "red"
      ? "bg-red"
      : mem?.state === "amber"
        ? "bg-accent-2"
        : "bg-green";

  return (
    <header className="border-b border-line px-5 py-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="font-mono text-[10px] tracking-[0.2em] uppercase text-muted">
            Free disk
          </div>
          <div
            className={`tabular text-4xl font-bold leading-none tracking-tight ${pctColor}`}
          >
            {disk ? formatPercent(pct) : "—"}
          </div>
          <div className="tabular mt-1 font-mono text-[11px] text-fg-2">
            {disk
              ? `${formatBytes(disk.free_bytes)} of ${formatBytes(disk.total_bytes)}`
              : "—"}
          </div>
        </div>

        <div className="flex flex-col items-end gap-2">
          <button
            onClick={() => {
              refresh();
              scan();
            }}
            className="rounded-md border border-line bg-surface px-2 py-1 text-fg-2 hover:bg-line"
            title="Re-scan"
          >
            <RotateCw size={14} />
          </button>
          <div className="flex items-center gap-2 font-mono text-[10px] tracking-[0.2em] uppercase text-muted">
            <span className={`block size-2 rounded-full ${memColor}`} />
            mem · {mem?.state ?? "—"}
          </div>
        </div>
      </div>
    </header>
  );
}
