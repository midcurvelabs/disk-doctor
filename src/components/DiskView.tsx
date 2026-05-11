import { useApp } from "@/store";
import { DiskSummary } from "@/components/DiskSummary";
import { Idle } from "@/components/Idle";
import { Results } from "@/components/Results";
import { Cleaning } from "@/components/Cleaning";
import { Done } from "@/components/Done";

export function DiskView() {
  const phase = useApp((s) => s.phase);
  return (
    <div className="flex h-full flex-col">
      <DiskSummary />
      <div className="flex-1 overflow-hidden">
        {phase === "idle" && <Idle />}
        {phase === "scanning" && (
          <div className="flex h-full items-center justify-center text-fg-2">
            Scanning your caches…
          </div>
        )}
        {phase === "results" && <Results />}
        {phase === "cleaning" && <Cleaning />}
        {phase === "done" && <Done />}
      </div>
    </div>
  );
}
