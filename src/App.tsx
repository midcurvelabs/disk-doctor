import { useEffect } from "react";
import { useApp } from "@/store";
import { Toolbar } from "@/components/Toolbar";
import { DiskView } from "@/components/DiskView";
import { MemoryView } from "@/components/MemoryView";

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

  return (
    <div className="flex h-screen w-screen flex-col bg-bg text-fg">
      <Toolbar />
      <main className="flex-1 overflow-hidden">
        {tab === "disk" ? <DiskView /> : <MemoryView />}
      </main>
    </div>
  );
}
