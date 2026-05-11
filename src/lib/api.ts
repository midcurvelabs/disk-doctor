import { invoke } from "@tauri-apps/api/core";

export type DiskStatus = {
  total_bytes: number;
  free_bytes: number;
  used_bytes: number;
  free_percent: number;
};

/** Activity-Monitor-style memory breakdown. All bytes. */
export type MemPressure = {
  state: "green" | "amber" | "red";
  total_bytes: number;
  app_bytes: number;
  wired_bytes: number;
  compressed_bytes: number;
  cached_files_bytes: number;
  available_bytes: number;
  swap_used_bytes: number;
  swap_total_bytes: number;
};

export type Safety = "Safe" | "Risky" | "DoNotKill";

export type ProcessRow = {
  pid: number;
  name: string;
  rss_bytes: number;
  process_count: number;
  safety: Safety;
  safety_reason: string;
  is_app_bundle: boolean;
};

export type Tier = 1 | 2 | 3;

export type CategoryResult = {
  id: string;
  label: string;
  tier: Tier;
  size_bytes: number;
  paths: string[];
  why: string;
  regenerates: string;
  running_app_block: string | null;
};

export type CleanRecord = {
  timestamp: string;
  category_ids: string[];
  recovered_bytes: number;
};

export type CleanResult = {
  recovered_bytes: number;
  per_category: { id: string; recovered_bytes: number; error?: string }[];
};

export const api = {
  diskStatus: () => invoke<DiskStatus>("disk_status"),
  memoryPressure: () => invoke<MemPressure>("memory_pressure"),
  topProcesses: (limit: number = 20) =>
    invoke<ProcessRow[]>("top_processes", { limit }),
  scanAll: () => invoke<CategoryResult[]>("scan_all"),
  scanOne: (id: string) => invoke<CategoryResult>("scan_one", { id }),
  clean: (ids: string[], dryRun = false) =>
    invoke<CleanResult>("clean", { ids, dryRun }),
  history: () => invoke<CleanRecord[]>("history"),
  openInFinder: (path: string) => invoke<void>("open_in_finder", { path }),
  openActivityMonitor: () => invoke<void>("open_activity_monitor"),
  quitProcess: (name: string, isAppBundle: boolean) =>
    invoke<number>("quit_process", { name, isAppBundle }),
};
