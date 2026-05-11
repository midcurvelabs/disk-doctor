/**
 * Window-top title bar that respects the macOS "Overlay" titleBarStyle.
 * Empty space gives room for the traffic-light buttons; the centered title
 * mirrors how Activity Monitor / System Settings present their window names.
 */
export function TitleBar() {
  return (
    <div
      data-tauri-drag-region
      className="flex h-7 items-center justify-center border-b border-line bg-bg-2 text-[12px] font-medium text-fg-2 select-none"
    >
      Disk Doctor
    </div>
  );
}
