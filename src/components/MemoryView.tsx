import { useEffect, useRef, useState } from "react";
import { Activity, X, Info } from "lucide-react";
import { useApp } from "@/store";
import { api, ProcessRow, MemPressure, Safety } from "@/lib/api";
import { formatBytes } from "@/lib/utils";

export function MemoryView() {
  const mem = useApp((s) => s.mem);
  const [procs, setProcs] = useState<ProcessRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [showExplainer, setShowExplainer] = useState(false);

  useEffect(() => {
    let alive = true;
    const tick = async () => {
      try {
        const list = await api.topProcesses(15);
        if (alive) {
          setProcs(list);
          setLoading(false);
        }
      } catch {
        if (alive) setLoading(false);
      }
    };
    tick();
    const t = setInterval(tick, 3000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, []);

  return (
    <div className="flex h-full flex-col">
      <PressureBlock mem={mem} onInfo={() => setShowExplainer((v) => !v)} />

      {showExplainer && (
        <div className="mx-3 mb-2 rounded-lg bg-surface px-3 py-2.5 text-[11.5px] leading-relaxed text-fg-2">
          macOS uses all available RAM as a cache by design. The number that
          matters is{" "}
          <span className="text-fg">memory pressure</span>, not "% used."
          Pressure is{" "}
          <span className="text-green">green</span> when there's plenty of
          reclaimable memory, <span className="text-orange">amber</span> when
          apps start competing, <span className="text-red">red</span> when the
          kernel starts killing processes.
        </div>
      )}

      <Callout mem={mem} />

      <div className="flex items-end justify-between border-t border-line px-4 pb-1.5 pt-3">
        <div>
          <div className="text-[12px] font-medium text-fg">Top apps by RAM</div>
          <div className="text-[11px] text-fg-3">
            Grouped by app · hover the <Info size={9} className="inline -mt-0.5" /> for what's safe to quit
          </div>
        </div>
        <button
          onClick={() => api.openActivityMonitor()}
          className="flex items-center gap-1.5 text-[11.5px] text-blue hover:text-blue-2"
        >
          <Activity size={11} />
          Open Activity Monitor
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-3 pb-3">
        {loading ? (
          <div className="px-4 py-6 text-[12px] text-fg-3">Reading…</div>
        ) : (
          <div className="card">
            {procs.map((p, i) => (
              <ProcRow
                key={`${p.name}-${i}`}
                p={p}
                totalBytes={mem?.total_bytes ?? 1}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function PressureBlock({
  mem,
  onInfo,
}: {
  mem: MemPressure | null;
  onInfo: () => void;
}) {
  if (!mem) {
    return (
      <section className="px-4 pt-3 pb-3">
        <div className="h-2 w-full animate-pulse rounded-full bg-surface" />
      </section>
    );
  }
  const used = mem.app_bytes + mem.wired_bytes + mem.compressed_bytes;
  const usedPct = (used / Math.max(mem.total_bytes, 1)) * 100;
  return (
    <section className="px-4 pt-3 pb-3">
      <div className="mb-2 flex items-baseline justify-between">
        <div className="flex items-center gap-1.5">
          <span className={`block size-1.5 rounded-full ${dotBg(mem.state)}`} />
          <span className="text-[12.5px] font-medium text-fg">
            Memory pressure:{" "}
            <span className={dotText(mem.state)}>{mem.state}</span>
          </span>
          <button
            onClick={onInfo}
            className="ml-1 text-fg-3 hover:text-fg-2"
            title="What does this mean?"
          >
            <Info size={11} />
          </button>
        </div>
        <span className="tabular text-[11px] text-fg-2">
          {formatBytes(mem.available_bytes)} available
        </span>
      </div>

      <div className="relative flex h-1.5 w-full overflow-hidden rounded-full bg-surface">
        <Seg pct={(mem.app_bytes / mem.total_bytes) * 100} className="bg-blue" />
        <Seg
          pct={(mem.wired_bytes / mem.total_bytes) * 100}
          className="bg-purple"
        />
        <Seg
          pct={(mem.compressed_bytes / mem.total_bytes) * 100}
          className="bg-orange"
        />
        <Seg
          pct={(mem.cached_files_bytes / mem.total_bytes) * 100}
          className="bg-fg-3 opacity-30"
        />
      </div>

      <div className="mt-3 grid grid-cols-2 gap-2">
        <Pill color="blue" label="App memory" value={formatBytes(mem.app_bytes)} />
        <Pill color="purple" label="Wired" value={formatBytes(mem.wired_bytes)} />
        <Pill
          color="orange"
          label="Compressed"
          value={formatBytes(mem.compressed_bytes)}
        />
        <Pill
          color="mute"
          label="Cached files"
          value={formatBytes(mem.cached_files_bytes)}
        />
      </div>

      {mem.swap_used_bytes > 0 && (
        <div className="mt-2.5 flex items-center justify-between rounded-lg bg-surface px-3 py-2 text-[11.5px]">
          <span className="text-fg-2">Swap in use</span>
          <span className="tabular text-fg">
            {formatBytes(mem.swap_used_bytes)}
          </span>
        </div>
      )}

      <div className="mt-1.5 flex items-center justify-between text-[10.5px] text-fg-3">
        <span>{Math.round(usedPct)}% in use</span>
        <span>Total {formatBytes(mem.total_bytes)}</span>
      </div>
    </section>
  );
}

function Callout({ mem }: { mem: MemPressure | null }) {
  if (!mem) return null;
  if (mem.state === "green") return null;
  const isAmber = mem.state === "amber";
  return (
    <div
      className={`mx-3 mb-2 rounded-lg px-3 py-2.5 text-[12px] leading-snug ${
        isAmber ? "bg-orange/10 text-orange" : "bg-red/10 text-red"
      }`}
    >
      <div className={`font-medium ${isAmber ? "text-orange" : "text-red"}`}>
        {isAmber
          ? "Memory is getting tight."
          : "Memory is critically pressured."}
      </div>
      <div className={isAmber ? "text-orange/80" : "text-red/80"}>
        Hover the <Info size={10} className="inline -mt-0.5" /> on each row to
        see what it is and whether it's safe to quit.
      </div>
    </div>
  );
}

const SAFETY_META: Record<
  Safety,
  { label: string; dot: string; text: string }
> = {
  Safe: { label: "Safe to quit", dot: "bg-green", text: "text-green" },
  Risky: { label: "Be careful", dot: "bg-orange", text: "text-orange" },
  DoNotKill: { label: "Do not quit", dot: "bg-red", text: "text-red" },
};

function ProcRow({
  p,
  totalBytes,
}: {
  p: ProcessRow;
  totalBytes: number;
}) {
  const pct = (p.rss_bytes / totalBytes) * 100;
  const refreshMemory = useApp((s) => s.refreshMemory);
  const blocked = p.safety === "DoNotKill";
  return (
    <div className="row grid-cols-[1fr_72px_44px_18px_18px] gap-2 py-1.5">
      <div className="min-w-0">
        <div className="flex items-center gap-1.5">
          <span className="truncate text-[12.5px] text-fg">{p.name}</span>
          {p.process_count > 1 && (
            <span className="shrink-0 rounded-full bg-surface-2 px-1.5 py-px font-mono text-[10px] text-fg-2">
              ×{p.process_count}
            </span>
          )}
        </div>
        <div className="font-mono text-[10.5px] text-fg-3">
          {p.is_app_bundle ? "app" : "process"} · pid {p.pid}
        </div>
      </div>
      <div className="tabular text-right font-mono text-[12px] text-fg">
        {formatBytes(p.rss_bytes)}
      </div>
      <div className="tabular text-right font-mono text-[11px] text-fg-2">
        {pct.toFixed(1)}%
      </div>
      <InfoBadge safety={p.safety} reason={p.safety_reason} name={p.name} />
      <button
        onClick={async () => {
          if (blocked) return;
          if (confirm(`Quit “${p.name}”?`)) {
            await api.quitProcess(p.name, p.is_app_bundle);
            setTimeout(refreshMemory, 600);
          }
        }}
        disabled={blocked}
        className={`ml-auto ${
          blocked
            ? "cursor-default text-fg-3 opacity-30"
            : "text-fg-3 hover:text-red"
        }`}
        title={blocked ? "System process — can't quit" : `Quit ${p.name}`}
      >
        <X size={12} />
      </button>
    </div>
  );
}

function InfoBadge({
  safety,
  reason,
  name,
}: {
  safety: Safety;
  reason: string;
  name: string;
}) {
  const [open, setOpen] = useState(false);
  const closeTimer = useRef<number | null>(null);
  const meta = SAFETY_META[safety];

  const openNow = () => {
    if (closeTimer.current) {
      clearTimeout(closeTimer.current);
      closeTimer.current = null;
    }
    setOpen(true);
  };
  const closeSoon = () => {
    if (closeTimer.current) clearTimeout(closeTimer.current);
    closeTimer.current = window.setTimeout(() => setOpen(false), 120);
  };

  return (
    <div
      className="relative flex items-center justify-center"
      onMouseEnter={openNow}
      onMouseLeave={closeSoon}
    >
      <span className={`block size-1.5 rounded-full ${meta.dot}`} />
      {open && (
        <div className="pointer-events-none absolute right-full top-1/2 z-50 mr-2 w-[260px] -translate-y-1/2 rounded-lg border border-line-strong bg-bg-2 px-3 py-2.5 shadow-[0_8px_30px_rgba(0,0,0,0.5)]">
          <div className="mb-1 flex items-center justify-between">
            <span className="text-[12px] font-semibold text-fg">{name}</span>
            <span
              className={`flex items-center gap-1 text-[10.5px] uppercase tracking-wide ${meta.text}`}
            >
              <span className={`block size-1.5 rounded-full ${meta.dot}`} />
              {meta.label}
            </span>
          </div>
          <div className="text-[11.5px] leading-snug text-fg-2">{reason}</div>
        </div>
      )}
    </div>
  );
}

function Seg({ pct, className }: { pct: number; className: string }) {
  if (pct <= 0) return null;
  return (
    <div
      style={{ width: `${Math.max(pct, 0.5)}%` }}
      className={`h-full ${className}`}
    />
  );
}

function Pill({
  color,
  label,
  value,
}: {
  color: "blue" | "purple" | "orange" | "mute";
  label: string;
  value: string;
}) {
  const dot =
    color === "blue"
      ? "bg-blue"
      : color === "purple"
        ? "bg-purple"
        : color === "orange"
          ? "bg-orange"
          : "bg-fg-3 opacity-50";
  return (
    <div className="flex items-center gap-2 rounded-lg bg-surface px-3 py-1.5">
      <span className={`block size-1.5 rounded-full ${dot}`} />
      <div className="flex flex-1 items-baseline justify-between gap-2">
        <span className="text-[11px] text-fg-2">{label}</span>
        <span className="tabular text-[12px] font-medium text-fg">{value}</span>
      </div>
    </div>
  );
}

function dotBg(state: string) {
  return state === "red"
    ? "bg-red"
    : state === "amber"
      ? "bg-orange"
      : "bg-green";
}
function dotText(state: string) {
  return state === "red"
    ? "text-red"
    : state === "amber"
      ? "text-orange"
      : "text-green";
}
