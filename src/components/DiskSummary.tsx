import { useApp } from "@/store";
import { formatBytes, formatPercent } from "@/lib/utils";

/**
 * Activity-Monitor-style summary: a single horizontal bar segmented into
 * used vs free, with stat columns below. Restrained, no orange.
 */
export function DiskSummary() {
  const disk = useApp((s) => s.disk);
  if (!disk) {
    return (
      <section className="px-4 pt-3 pb-4">
        <div className="h-2 w-full animate-pulse rounded-full bg-surface" />
      </section>
    );
  }
  const usedPct = 100 - disk.free_percent;
  return (
    <section className="px-4 pt-3 pb-4">
      <div className="mb-2 flex items-baseline justify-between text-[11px] text-fg-2">
        <span>Macintosh HD — Data</span>
        <span className="tabular">
          {formatPercent(disk.free_percent, 1)} free
        </span>
      </div>
      <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-surface">
        <div
          className="absolute inset-y-0 left-0 bg-blue"
          style={{ width: `${usedPct}%` }}
        />
      </div>
      <div className="mt-3 grid grid-cols-3 gap-3">
        <Stat label="Used" value={formatBytes(disk.used_bytes)} />
        <Stat label="Free" value={formatBytes(disk.free_bytes)} accent="green" />
        <Stat label="Total" value={formatBytes(disk.total_bytes)} />
      </div>
    </section>
  );
}

function Stat({
  label,
  value,
  accent,
}: {
  label: string;
  value: string;
  accent?: "green" | "red";
}) {
  return (
    <div className="rounded-lg bg-surface px-3 py-2">
      <div className="text-[10px] uppercase tracking-wide text-fg-3">
        {label}
      </div>
      <div
        className={`tabular text-[15px] font-medium ${
          accent === "green"
            ? "text-green"
            : accent === "red"
              ? "text-red"
              : "text-fg"
        }`}
      >
        {value}
      </div>
    </div>
  );
}
