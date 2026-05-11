import { useEffect } from "react";
import { useApp } from "@/store";
import { Header } from "@/components/Header";
import { Idle } from "@/components/Idle";
import { Results } from "@/components/Results";
import { Cleaning } from "@/components/Cleaning";
import { Done } from "@/components/Done";

export default function App() {
  const phase = useApp((s) => s.phase);
  const refreshStatus = useApp((s) => s.refreshStatus);

  useEffect(() => {
    refreshStatus();
    const t = setInterval(refreshStatus, 30_000);
    return () => clearInterval(t);
  }, [refreshStatus]);

  return (
    <div className="flex h-screen w-screen flex-col bg-bg text-fg">
      <Header />
      <main className="flex-1 overflow-y-auto">
        {phase === "idle" && <Idle />}
        {phase === "scanning" && (
          <div className="flex h-full items-center justify-center text-muted">
            Scanning…
          </div>
        )}
        {phase === "results" && <Results />}
        {phase === "cleaning" && <Cleaning />}
        {phase === "done" && <Done />}
      </main>
    </div>
  );
}
