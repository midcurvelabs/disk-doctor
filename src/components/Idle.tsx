import { useApp } from "@/store";

export function Idle() {
  const scan = useApp((s) => s.scan);
  return (
    <div className="flex h-full flex-col items-center justify-center gap-5 px-8 text-center">
      <div className="max-w-[36ch] text-[13px] leading-relaxed text-fg-2">
        Audit caches that generic cleaners miss — Claude Code VMs, opencode
        snapshots, <span className="font-mono text-[12px]">node_modules</span>,
        Ollama, Electron partitions.
      </div>
      <button
        onClick={scan}
        className="rounded-md bg-blue px-4 py-1.5 text-[13px] font-medium text-white hover:bg-blue-2 active:opacity-80"
      >
        Scan
      </button>
      <div className="text-[11px] text-fg-3">
        Read-only. Nothing is deleted without your click.
      </div>
    </div>
  );
}
