import { useApp, type Tab } from "@/store";
import { cn } from "@/lib/utils";

const TABS: { id: Tab; label: string }[] = [
  { id: "disk", label: "Disk" },
  { id: "memory", label: "Memory" },
];

export function Tabs() {
  const tab = useApp((s) => s.tab);
  const setTab = useApp((s) => s.setTab);
  return (
    <div className="flex items-center justify-center bg-bg-2 px-3 pb-2 pt-1">
      <div className="inline-flex rounded-lg bg-surface p-0.5">
        {TABS.map((t) => (
          <button
            key={t.id}
            onClick={() => setTab(t.id)}
            className={cn(
              "min-w-[88px] rounded-md px-3 py-1 text-[12px] font-medium transition-colors",
              tab === t.id
                ? "bg-surface-2 text-fg shadow-[0_1px_0_rgba(255,255,255,0.05),0_1px_2px_rgba(0,0,0,0.3)]"
                : "text-fg-2 hover:text-fg"
            )}
          >
            {t.label}
          </button>
        ))}
      </div>
    </div>
  );
}
