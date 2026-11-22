import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useApp } from "@/store";
import { Toolbar } from "@/components/Toolbar";
import { DiskView } from "@/components/DiskView";
import { MemoryView } from "@/components/MemoryView";
import { PipelineView } from "@/components/PipelineView";

export default function App() {
  const refreshStatus = useApp((s) => s.refreshStatus);
  const refreshMemory = useApp((s) => s.refreshMemory);
  const tab = useApp((s) => s.tab);

  useEffect(() => {
    refreshStatus();
    refreshMemory();
    const t1 = setInterval(refreshStatus, 30_000);
    const t2 = setInterval(refreshMemory, 3_000);
    return () => {
      clearInterval(t1);
      clearInterval(t2);
    };
  }, [refreshStatus, refreshMemory]);

  // The window is only hidden when closed, so state lives on. When it comes
  // back: refresh the disk numbers at once, and rescan the cache list if it
  // is older than STALE_MS. The tray "Scan now" item forces a rescan.
  useEffect(() => {
    const STALE_MS = 5 * 60_000;
    const unlisteners: Array<() => void> = [];
    getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (!focused) return;
        const s = useApp.getState();
        s.refreshStatus();
        s.refreshMemory();
        if (
          s.tab === "disk" &&
          s.phase === "results" &&
          (!s.scannedAt || Date.now() - s.scannedAt > STALE_MS)
        ) {
          s.rescan();
        }
      })
      .then((u) => unlisteners.push(u));
    listen("trigger-scan", () => {
      const s = useApp.getState();
      s.setTab("disk");
      s.rescan();
    }).then((u) => unlisteners.push(u));
    return () => unlisteners.forEach((u) => u());
  }, []);

  return (
    <div className="flex h-screen w-screen flex-col bg-bg text-fg">
      <Toolbar />
      <main className="flex-1 overflow-hidden">
        {tab === "disk" ? <DiskView /> : tab === "pipeline" ? <PipelineView /> : <MemoryView />}
      </main>
    </div>
  );
}
