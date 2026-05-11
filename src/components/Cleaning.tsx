export function Cleaning() {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 px-8 text-center">
      <div className="size-5 animate-spin rounded-full border-2 border-fg-3 border-t-blue" />
      <div className="text-[13px] text-fg-2">Cleaning…</div>
      <div className="text-[11px] text-fg-3">Don't quit yet.</div>
    </div>
  );
}
