import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  AlertCircle,
  ChevronDown,
  ChevronRight,
  FolderOpen,
  HardDrive,
  Settings2,
} from "lucide-react";
import { useApp } from "@/store";
import { api, PipelineProgress, PipelineRule, Tier } from "@/lib/api";
import { formatBytes } from "@/lib/utils";
import { DiskSummary } from "@/components/DiskSummary";

const TIER_META: Record<Tier, { label: string; sub: string; dot: string }> = {
  1: {
    label: "Safe to clean",
    sub: "Regenerates, or temp files nobody touched for days.",
    dot: "bg-green",
  },
  2: {
    label: "Back up first",
    sub: "Copied to the backup drive and verified before removal.",
    dot: "bg-orange",
  },
  3: {
    label: "Your call",
    sub: "History you may still want.",
    dot: "bg-red",
  },
};

function RuleRow({ r, backupMounted }: { r: PipelineRule; backupMounted: boolean }) {
  const selected = useApp((s) => s.pipeSelected.has(r.id));
  const toggle = useApp((s) => s.pipeToggle);
  const [open, setOpen] = useState(false);
  const needsDrive = r.action === "offload" && !backupMounted;
  const disabled = r.ready_bytes === 0 || needsDrive;
  const ready = r.items.filter((i) => i.eligible);
  const waiting = r.items.filter((i) => !i.eligible);

  return (
    <div className={selected ? "bg-surface-2" : ""}>
      <div
        onClick={() => !disabled && toggle(r.id)}
        className={`row grid-cols-[18px_14px_1fr_20px_72px] gap-2.5 ${
          disabled ? "opacity-50" : "cursor-default hover:bg-surface-2"
        }`}
      >
        <input
          type="checkbox"
          checked={selected}
          disabled={disabled}
          onChange={() => toggle(r.id)}
          onClick={(e) => e.stopPropagation()}
          className="size-3.5"
        />
        <button
          onClick={(e) => {
            e.stopPropagation();
            setOpen(!open);
          }}
          className="text-fg-3 hover:text-fg"
          title={open ? "Hide items" : "Show items"}
        >
          {open ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
        </button>
        <div className="min-w-0">
          <div className="truncate text-[12.5px] text-fg">{r.label}</div>
          <div className="truncate text-[11px] text-fg-3">{r.why}</div>
          {r.waiting_bytes > 0 && (
            <div className="text-[11px] text-fg-3">
              <span className="tabular">{formatBytes(r.waiting_bytes)}</span> more
              once in-progress work is finished
            </div>
          )}
          {needsDrive && r.ready_bytes > 0 && (
            <div className="mt-0.5 inline-flex items-center gap-1 text-[11px] text-orange">
              <AlertCircle size={11} />
              Plug in the backup drive to offload
            </div>
          )}
          {r.error && <div className="text-[11px] text-red">{r.error}</div>}
        </div>
        <button
          onClick={(e) => {
            e.stopPropagation();
            api.openInFinder(r.root);
          }}
          className="text-fg-3 hover:text-fg"
          title="Reveal in Finder"
        >
          <FolderOpen size={13} />
        </button>
        <div className="tabular text-right font-mono text-[12px] text-fg">
          {formatBytes(r.ready_bytes)}
        </div>
      </div>
      {open && (
        <div className="border-t border-line pb-1.5 pl-[50px] pr-3 pt-1">
          {[...ready, ...waiting].slice(0, 40).map((i) => (
            <div
              key={i.path}
              className={`grid grid-cols-[1fr_auto_64px] items-center gap-2 py-0.5 text-[11px] ${
                i.eligible ? "text-fg-2" : "text-fg-3"
              }`}
            >
              <button
                onClick={() => api.openInFinder(i.path)}
                className="truncate text-left hover:text-fg"
                title={i.path}
              >
                {i.name}
              </button>
              <span className="truncate">{i.reason}</span>
              <span className="tabular text-right font-mono">{formatBytes(i.size_bytes)}</span>
            </div>
          ))}
          {r.items.length > 40 && (
            <div className="py-0.5 text-[11px] text-fg-3">and {r.items.length - 40} more</div>
          )}
          {r.items.length === 0 && (
            <div className="py-0.5 text-[11px] text-fg-3">Nothing here right now.</div>
          )}
        </div>
      )}
    </div>
  );
}

function Working({ p }: { p: PipelineProgress | null }) {
  const pct = p && p.total_bytes > 0 ? Math.min(100, (p.done_bytes / p.total_bytes) * 100) : 0;
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 px-8">
      <div className="text-[13px] text-fg">{p?.phase ?? "Starting"}…</div>
      <div className="h-1.5 w-full overflow-hidden rounded-full bg-surface">
        <div className="h-full bg-blue transition-[width]" style={{ width: `${pct}%` }} />
      </div>
      <div className="flex w-full justify-between text-[11px] text-fg-3">
        <span className="truncate pr-3">{p?.current}</span>
        <span className="tabular shrink-0">
          {p ? `${formatBytes(p.done_bytes)} of ${formatBytes(p.total_bytes)}` : ""}
        </span>
      </div>
    </div>
  );
}

function DoneView() {
  const r = useApp((s) => s.pipeResult);
  const back = useApp((s) => s.pipeBack);
  const emptyTrash = useApp((s) => s.pipeEmptyTrash);
  if (!r) return null;
  const errors = r.rules.flatMap((x) => x.errors);
  const already = r.rules.reduce((s, x) => s + x.already_backed_bytes, 0);
  return (
    <div className="flex h-full flex-col">
      <div className="flex-1 overflow-y-auto px-5 pt-6">
        <div className="tabular text-[28px] font-semibold text-fg">{formatBytes(r.trashed_bytes)}</div>
        <div className="text-[12px] text-fg-2">moved to the Trash</div>
        {r.copied_bytes > 0 && (
          <div className="mt-3 text-[12px] text-fg-2">
            <span className="tabular text-fg">{formatBytes(r.copied_bytes)}</span> copied to the backup
            drive and verified byte for byte.
          </div>
        )}
        {already > 0 && (
          <div className="mt-1 text-[12px] text-fg-2">
            <span className="tabular text-fg">{formatBytes(already)}</span> already had an identical copy
            there.
          </div>
        )}
        <div className="mt-3 text-[12px] text-fg-2">
          The space comes back when the Trash is emptied (
          <span className="tabular">{formatBytes(r.trash_bytes_after)}</span> in it now).
        </div>
        {errors.length > 0 && (
          <div className="mt-4 rounded-md bg-surface p-2.5 text-[11px] text-orange">
            <div className="mb-1 font-medium">Kept {errors.length} file(s):</div>
            {errors.slice(0, 8).map((e) => (
              <div key={e} className="truncate" title={e}>
                {e}
              </div>
            ))}
          </div>
        )}
      </div>
      <footer className="flex items-center gap-2 border-t border-line bg-bg-2 px-3 py-2.5">
        <button
          onClick={async () => {
            await emptyTrash();
            back();
          }}
          className="flex flex-1 items-center justify-center rounded-md bg-blue px-3 py-1.5 text-[12.5px] font-medium text-white hover:bg-blue-2"
        >
          Empty Trash · {formatBytes(r.trash_bytes_after)}
        </button>
        <button onClick={back} className="rounded-md px-2 py-1.5 text-[12px] text-fg-2 hover:bg-surface">
          Done
        </button>
      </footer>
    </div>
  );
}

export function PipelineView() {
  const pipe = useApp((s) => s.pipe);
  const phase = useApp((s) => s.pipePhase);
  const selected = useApp((s) => s.pipeSelected);
  const progress = useApp((s) => s.pipeProgress);
  const error = useApp((s) => s.pipeError);
  const scan = useApp((s) => s.pipeScan);
  const run = useApp((s) => s.pipeRun);
  const emptyTrash = useApp((s) => s.pipeEmptyTrash);
  const setProgress = useApp((s) => s.pipeSetProgress);
  const refreshStatus = useApp((s) => s.refreshStatus);

  useEffect(() => {
    if (phase === "idle") scan();
    const un = listen<PipelineProgress>("pipeline-progress", (e) => setProgress(e.payload));
    return () => {
      un.then((f) => f());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const rules = pipe?.rules ?? [];
  const byTier = (t: Tier) =>
    rules.filter((r) => r.tier === t).sort((a, b) => b.ready_bytes - a.ready_bytes);
  const selectedRules = rules.filter((r) => selected.has(r.id));
  const selectedTotal = selectedRules.reduce((s, r) => s + r.ready_bytes, 0);
  const offloads = selectedRules.some((r) => r.action === "offload");
  const mounted = !!pipe?.backup.mounted;

  return (
    <div className="flex h-full flex-col">
      <DiskSummary />
      <div className="flex items-center justify-between border-b border-line px-4 py-1.5 text-[11px] text-fg-3">
        <span className="inline-flex items-center gap-1.5">
          <HardDrive size={12} className={mounted ? "text-green" : "text-fg-3"} />
          {mounted
            ? `Backup drive connected · ${formatBytes(pipe!.backup.free_bytes)} free`
            : "Backup drive not connected"}
        </span>
        <span className="inline-flex items-center gap-2">
          <label className="inline-flex items-center gap-1" title="Warn me when free space drops below this">
            Warn under
            <select
              value={pipe?.threshold_gb ?? 30}
              onChange={async (e) => {
                await api.pipelineSetThreshold(Number(e.target.value));
                await refreshStatus();
                scan();
              }}
              className="rounded bg-surface px-1 py-0.5 text-fg"
            >
              {[15, 20, 30, 40, 50, 75, 100].map((n) => (
                <option key={n} value={n}>
                  {n} GB
                </option>
              ))}
            </select>
          </label>
          <button onClick={() => api.pipelineEditRules()} className="hover:text-fg" title="Edit the rules (pipeline.json)">
            <Settings2 size={12} />
          </button>
        </span>
      </div>

      <div className="flex-1 overflow-hidden">
        {phase === "scanning" && !pipe && (
          <div className="flex h-full items-center justify-center text-fg-2">Checking your pipeline folders…</div>
        )}
        {phase === "working" && <Working p={progress} />}
        {phase === "done" && <DoneView />}
        {(phase === "results" || (phase === "scanning" && pipe) || phase === "idle") && pipe && (
          <div className="flex h-full flex-col">
            <div className="flex-1 overflow-y-auto pb-3 pt-1">
              {error && <div className="mx-4 mb-2 text-[11px] text-red">{error}</div>}
              {([1, 2, 3] as Tier[]).map((t) => {
                const list = byTier(t);
                if (list.length === 0) return null;
                const meta = TIER_META[t];
                const total = list.reduce((s, r) => s + r.ready_bytes, 0);
                return (
                  <section key={t} className="mb-3 first:mt-1">
                    <header className="flex items-end justify-between px-4 pb-1.5">
                      <div className="flex items-center gap-2">
                        <span className={`block size-1.5 rounded-full ${meta.dot}`} />
                        <div>
                          <div className="text-[12px] font-medium text-fg">{meta.label}</div>
                          <div className="text-[11px] text-fg-3">{meta.sub}</div>
                        </div>
                      </div>
                      <div className="tabular text-[11px] text-fg-3">{formatBytes(total)} ready</div>
                    </header>
                    <div className="card mx-3">
                      {list.map((r) => (
                        <RuleRow key={r.id} r={r} backupMounted={mounted} />
                      ))}
                    </div>
                  </section>
                );
              })}
            </div>
            <footer className="flex flex-col gap-1.5 border-t border-line bg-bg-2 px-3 py-2.5">
              <div className="flex items-center gap-2">
                <button
                  onClick={run}
                  disabled={selectedTotal === 0}
                  className="flex flex-1 items-center justify-center rounded-md bg-blue px-3 py-1.5 text-[12.5px] font-medium text-white hover:bg-blue-2 disabled:opacity-30"
                >
                  {selectedTotal === 0
                    ? "Select what to clean"
                    : `${offloads ? "Back up and clean" : "Clean selected"} · ${formatBytes(selectedTotal)}`}
                </button>
                <button
                  onClick={scan}
                  className="rounded-md px-2 py-1.5 text-[12px] text-fg-2 hover:bg-surface"
                >
                  Re-scan
                </button>
              </div>
              {pipe.trash_bytes > 0 && (
                <button
                  onClick={emptyTrash}
                  className="text-left text-[11px] text-fg-3 hover:text-fg"
                >
                  Trash holds <span className="tabular">{formatBytes(pipe.trash_bytes)}</span>. Empty it to get
                  the space back.
                </button>
              )}
            </footer>
          </div>
        )}
      </div>
    </div>
  );
}
