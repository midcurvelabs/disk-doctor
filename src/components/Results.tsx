import { Trash2, FolderOpen, AlertTriangle } from "lucide-react";
import { useApp } from "@/store";
import { api, CategoryResult, Tier } from "@/lib/api";
import { formatBytes } from "@/lib/utils";

const TIER_META: Record<Tier, { label: string; sub: string; accent: string }> =
  {
    1: {
      label: "Safe to clean",
      sub: "Pure caches that regenerate automatically.",
      accent: "text-green",
    },
    2: {
      label: "Confirm first",
      sub: "Larger artifacts that rebuild on next use.",
      accent: "text-accent-2",
    },
    3: {
      label: "Your call",
      sub: "Apps you might want to keep around.",
      accent: "text-red",
    },
  };

function Row({ c }: { c: CategoryResult }) {
  const selected = useApp((s) => s.selectedIds.has(c.id));
  const toggle = useApp((s) => s.toggleSelect);
  const blocked = !!c.running_app_block;
  return (
    <div
      className={`flex items-center gap-3 border-t border-line px-3 py-2.5 hover:bg-surface ${selected ? "bg-surface" : ""}`}
    >
      <input
        type="checkbox"
        checked={selected}
        disabled={blocked || c.size_bytes === 0}
        onChange={() => toggle(c.id)}
        className="size-4 accent-[var(--color-accent)]"
      />
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm text-fg">{c.label}</div>
        <div className="truncate font-mono text-[10px] text-muted">
          {c.why}
        </div>
        {blocked && (
          <div className="mt-1 inline-flex items-center gap-1 font-mono text-[10px] uppercase tracking-wider text-red">
            <AlertTriangle size={10} />
            {c.running_app_block} is open — quit it first
          </div>
        )}
      </div>
      <button
        onClick={() => c.paths[0] && api.openInFinder(c.paths[0])}
        className="text-muted hover:text-fg"
        title="Open in Finder"
      >
        <FolderOpen size={14} />
      </button>
      <div className="tabular w-20 text-right font-mono text-sm text-fg">
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
    <section className="mt-3 first:mt-0">
      <header className="flex items-baseline justify-between px-3 pb-1">
        <div>
          <div
            className={`font-mono text-[10px] tracking-[0.2em] uppercase ${meta.accent}`}
          >
            Tier {tier} · {meta.label}
          </div>
          <div className="text-[11px] text-muted">{meta.sub}</div>
        </div>
        <button
          onClick={() => selectTier(tier)}
          className="font-mono text-[10px] uppercase tracking-[0.15em] text-muted hover:text-fg"
        >
          select all · {formatBytes(total)}
        </button>
      </header>
      <div>
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
      <div className="flex-1 overflow-y-auto px-2 pb-4 pt-2">
        <Section tier={1} categories={byTier(1)} />
        <Section tier={2} categories={byTier(2)} />
        <Section tier={3} categories={byTier(3)} />
      </div>
      <footer className="flex items-center gap-2 border-t border-line bg-bg-2 px-3 py-3">
        {hasSelection ? (
          <>
            <button
              onClick={cleanSelected}
              className="flex flex-1 items-center justify-center gap-2 rounded-md bg-accent px-3 py-2 text-sm font-medium text-bg"
            >
              <Trash2 size={14} />
              Clean selected · {formatBytes(selectedTotal)}
            </button>
            <button
              onClick={clear}
              className="rounded-md border border-line px-3 py-2 text-sm text-fg-2 hover:bg-surface"
            >
              clear
            </button>
          </>
        ) : (
          <button
            onClick={cleanSafe}
            disabled={safeTotal === 0}
            className="flex flex-1 items-center justify-center gap-2 rounded-md bg-accent px-3 py-2 text-sm font-medium text-bg disabled:opacity-40"
          >
            <Trash2 size={14} />
            Clean safe stuff · {formatBytes(safeTotal)}
          </button>
        )}
      </footer>
    </div>
  );
}
