import { useApp, type Tab } from "@/store";
import { cn } from "@/lib/utils";

const TABS: { id: Tab; label: string }[] = [
  { id: "disk", label: "Disk" },
  { id: "memory", label: "Memory" },
];

/**
 * Single macOS-style toolbar that owns the entire window chrome.
 * - Draggable across its whole surface (Tauri auto-excludes <button>).
 * - Tall enough (44px) to grab without aiming.
 * - Left padding leaves room for the native traffic-light buttons.
 * - The segmented control sits on the right, matching Settings.app.
 */
export function Toolbar() {
  const tab = useApp((s) => s.tab);
  const setTab = useApp((s) => s.setTab);
  return (
    <div
      data-tauri-drag-region
      className="flex h-11 shrink-0 items-center justify-between border-b border-line bg-bg-2 pl-[88px] pr-3"
    >
      <div
        data-tauri-drag-region
        className="text-[13px] font-semibold tracking-tight text-fg"
      >
        Disk Doctor
      </div>
      <div className="inline-flex rounded-lg bg-surface p-0.5">
        {TABS.map((t) => (
          <button
            key={t.id}
            onClick={() => setTab(t.id)}
            className={cn(
              "min-w-[78px] rounded-md px-3 py-1 text-[12px] font-medium transition-colors",
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
