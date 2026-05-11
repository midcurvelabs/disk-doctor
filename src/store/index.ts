import { create } from "zustand";
import {
  api,
  CategoryResult,
  CleanResult,
  DiskStatus,
  MemPressure,
} from "@/lib/api";

type Phase = "idle" | "scanning" | "results" | "cleaning" | "done";
export type Tab = "disk" | "memory";

interface AppState {
  tab: Tab;
  setTab: (t: Tab) => void;

  disk: DiskStatus | null;
  mem: MemPressure | null;
  categories: CategoryResult[];
  selectedIds: Set<string>;
  phase: Phase;
  lastCleanResult: CleanResult | null;
  error: string | null;

  refreshStatus: () => Promise<void>;
  refreshMemory: () => Promise<void>;
  scan: () => Promise<void>;
  toggleSelect: (id: string) => void;
  selectTier: (tier: 1 | 2 | 3) => void;
  clear: () => void;
  cleanSelected: () => Promise<void>;
  cleanSafe: () => Promise<void>;
}

export const useApp = create<AppState>((set, get) => ({
  tab: "disk",
  setTab: (tab) => set({ tab }),

  disk: null,
  mem: null,
  categories: [],
  selectedIds: new Set(),
  phase: "idle",
  lastCleanResult: null,
  error: null,

  refreshStatus: async () => {
    try {
      const disk = await api.diskStatus();
      set({ disk, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  refreshMemory: async () => {
    try {
      const mem = await api.memoryPressure();
      set({ mem });
    } catch {
      /* swallow — UI shows em-dash */
    }
  },

  scan: async () => {
    set({ phase: "scanning", error: null });
    try {
      const categories = await api.scanAll();
      set({ categories, phase: "results" });
    } catch (e) {
      set({ phase: "idle", error: String(e) });
    }
  },

  toggleSelect: (id) => {
    const next = new Set(get().selectedIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    set({ selectedIds: next });
  },

  selectTier: (tier) => {
    const next = new Set(get().selectedIds);
    for (const c of get().categories) {
      if (c.tier === tier && c.size_bytes > 0) next.add(c.id);
    }
    set({ selectedIds: next });
  },

  clear: () => set({ selectedIds: new Set() }),

  cleanSelected: async () => {
    const ids = [...get().selectedIds];
    if (ids.length === 0) return;
    set({ phase: "cleaning", error: null });
    try {
      const result = await api.clean(ids, false);
      set({ phase: "done", lastCleanResult: result, selectedIds: new Set() });
      await get().refreshStatus();
      await get().scan();
    } catch (e) {
      set({ phase: "results", error: String(e) });
    }
  },

  cleanSafe: async () => {
    const ids = get()
      .categories.filter((c) => c.tier === 1 && c.size_bytes > 0)
      .map((c) => c.id);
    if (ids.length === 0) return;
    set({ phase: "cleaning", error: null });
    try {
      const result = await api.clean(ids, false);
      set({ phase: "done", lastCleanResult: result });
      await get().refreshStatus();
      await get().scan();
    } catch (e) {
      set({ phase: "results", error: String(e) });
    }
  },
}));
