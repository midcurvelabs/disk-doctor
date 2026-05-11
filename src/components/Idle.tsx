import { Search } from "lucide-react";
import { useApp } from "@/store";

export function Idle() {
  const scan = useApp((s) => s.scan);
  return (
    <div className="flex h-full flex-col items-center justify-center gap-6 p-8 text-center">
      <div className="font-mono text-[11px] tracking-[0.22em] uppercase text-accent">
        Disk Doctor
      </div>
      <div className="max-w-[36ch] text-lg leading-tight text-fg-2">
        Audit caches that generic Mac cleaners miss. Claude VMs, opencode
        snapshots, node_modules, Ollama, Electron partitions.
      </div>
      <button
        onClick={scan}
        className="flex items-center gap-2 rounded-lg bg-accent px-6 py-3 font-medium text-bg hover:opacity-90"
      >
        <Search size={16} />
        Scan my Mac
      </button>
      <div className="font-mono text-[10px] tracking-[0.18em] uppercase text-muted">
        Read-only. Nothing is deleted without your click.
      </div>
    </div>
  );
}
