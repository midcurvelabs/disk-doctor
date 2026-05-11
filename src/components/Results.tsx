import { FolderOpen, AlertCircle } from "lucide-react";
import { useApp } from "@/store";
import { api, CategoryResult, Tier } from "@/lib/api";
import { formatBytes } from "@/lib/utils";

const TIER_META: Record<
  Tier,
  { label: string; sub: string; dot: string }
> = {
  1: {
    label: "Safe to clean",
    sub: "Pure caches that regenerate automatically.",
    dot: "bg-green",
  },
  2: {
    label: "Confirm first",
    sub: "Rebuild on next use; verify the app is closed.",
    dot: "bg-orange",
  },
  3: {
    label: "Your call",
    sub: "User-judgment cleanups.",
    dot: "bg-red",
  },
};

function Row({ c }: { c: CategoryResult }) {
  const selected = useApp((s) => s.selectedIds.has(c.id));
  const toggle = useApp((s) => s.toggleSelect);
  const blocked = !!c.running_app_block;
  const empty = c.size_bytes === 0;
  return (
    <div
      onClick={() => !blocked && !empty && toggle(c.id)}
      className={`row grid-cols-[18px_1fr_auto_20px_72px] gap-2.5 ${
        !blocked && !empty
          ? "cursor-default hover:bg-surface-2"
          : "opacity-50"
      } ${selected ? "bg-surface-2" : ""}`}
    >
      <input
        type="checkbox"
        checked={selected}
        disabled={blocked || empty}
        onChange={() => toggle(c.id)}
        onClick={(e) => e.stopPropagation()}
        className="size-3.5"
      />
      <div className="min-w-0">
        <div className="truncate text-[12.5px] text-fg">{c.label}</div>
        <div className="truncate text-[11px] text-fg-3">{c.why}</div>
        {blocked && (
          <div className="mt-0.5 inline-flex items-center gap-1 text-[11px] text-orange">
            <AlertCircle size={11} />
            {c.running_app_block} is open — quit it first
          </div>
        )}
      </div>
      <div />
      <button
        onClick={(e) => {
          e.stopPropagation();
          c.paths[0] && api.openInFinder(c.paths[0]);
        }}
        className="text-fg-3 hover:text-fg"
        title="Reveal in Finder"
      >
        <FolderOpen size={13} />
      </button>
      <div className="tabular text-right font-mono text-[12px] text-fg">
        {formatBytes(c.size_bytes)}
      </div>
    </div>
  );
}

function Section({
  tier,
  categories,
}: {
  tier: Tier;
  categories: CategoryResult[];
}) {
  const selectTier = useApp((s) => s.selectTier);
  const total = categories.reduce((s, c) => s + c.size_bytes, 0);
  const meta = TIER_META[tier];
  if (categories.length === 0) return null;
  return (
    <section className="mb-3 first:mt-1">
      <header className="flex items-end justify-between px-4 pb-1.5">
        <div className="flex items-center gap-2">
          <span className={`block size-1.5 rounded-full ${meta.dot}`} />
          <div>
            <div className="text-[12px] font-medium text-fg">
              {meta.label}
            </div>
            <div className="text-[11px] text-fg-3">{meta.sub}</div>
          </div>
        </div>
        <button
          onClick={() => selectTier(tier)}
          className="text-[11px] font-medium text-blue hover:text-blue-2"
        >
          Select all · {formatBytes(total)}
        </button>
      </header>
      <div className="card mx-3">
        {categories.map((c) => (
          <Row key={c.id} c={c} />
        ))}
      </div>
    </section>
  );
}

export function Results() {
  const cats = useApp((s) => s.categories);
  const selectedIds = useApp((s) => s.selectedIds);
  const cleanSafe = useApp((s) => s.cleanSafe);
  const cleanSelected = useApp((s) => s.cleanSelected);
  const clear = useApp((s) => s.clear);

  const byTier = (t: Tier) =>
    cats
      .filter((c) => c.tier === t)
      .sort((a, b) => b.size_bytes - a.size_bytes);

  const selectedTotal = cats
    .filter((c) => selectedIds.has(c.id))
    .reduce((s, c) => s + c.size_bytes, 0);

  const safeTotal = cats
    .filter((c) => c.tier === 1 && c.size_bytes > 0)
    .reduce((s, c) => s + c.size_bytes, 0);

  const hasSelection = selectedIds.size > 0;

  return (
    <div className="flex h-full flex-col">
      <div className="flex-1 overflow-y-auto pt-1 pb-3">
        <Section tier={1} categories={byTier(1)} />
        <Section tier={2} categories={byTier(2)} />
        <Section tier={3} categories={byTier(3)} />
      </div>
      <footer className="flex items-center gap-2 border-t border-line bg-bg-2 px-3 py-2.5">
        {hasSelection ? (
          <>
            <button
              onClick={cleanSelected}
              className="flex flex-1 items-center justify-center rounded-md bg-blue px-3 py-1.5 text-[12.5px] font-medium text-white hover:bg-blue-2 active:opacity-80"
            >
              Clean selected · {formatBytes(selectedTotal)}
            </button>
            <button
              onClick={clear}
              className="rounded-md px-2 py-1.5 text-[12px] text-fg-2 hover:bg-surface"
            >
              Clear
            </button>
          </>
        ) : (
          <button
            onClick={cleanSafe}
            disabled={safeTotal === 0}
            className="flex flex-1 items-center justify-center rounded-md bg-blue px-3 py-1.5 text-[12.5px] font-medium text-white hover:bg-blue-2 disabled:opacity-30"
          >
            Clean safe stuff · {formatBytes(safeTotal)}
          </button>
        )}
      </footer>
    </div>
  );
}
