import { create } from "zustand";
import type { Project, SheetData, LogEntry, Preferences } from "./types";

interface AppStore {
  project: Project | null;
  sheets: SheetData[];
  log: LogEntry[];
  isPacking: boolean;
  isWatching: boolean;
  dirty: boolean;
  prefs: Preferences;
  prefsOpen: boolean;
  activeSheet: number;

  setProject: (p: Project) => void;
  setSheets: (s: SheetData[], log: LogEntry[]) => void;
  appendLog: (entry: LogEntry) => void;
  setLog: (entries: LogEntry[]) => void;
  setIsPacking: (v: boolean) => void;
  setIsWatching: (v: boolean) => void;
  setDirty: (v: boolean) => void;
  setPrefs: (p: Preferences) => void;
  setPrefsOpen: (v: boolean) => void;
  setActiveSheet: (i: number) => void;
}

const defaultPrefs: Preferences = {
  dark_mode: true,
  auto_check_updates: true,
  language: "En",
  ui_scale: 1.0,
};

export const useStore = create<AppStore>((set) => ({
  project: null,
  sheets: [],
  log: [],
  isPacking: false,
  isWatching: false,
  dirty: false,
  prefs: defaultPrefs,
  prefsOpen: false,
  activeSheet: 0,

  setProject: (p) => set({ project: p, dirty: false }),
  setSheets: (sheets, log) => set({ sheets, log, activeSheet: 0 }),
  appendLog: (entry) => set((s) => ({ log: [...s.log, entry] })),
  setLog: (log) => set({ log }),
  setIsPacking: (isPacking) => set({ isPacking }),
  setIsWatching: (isWatching) => set({ isWatching }),
  setDirty: (dirty) => set({ dirty }),
  setPrefs: (prefs) => set({ prefs }),
  setPrefsOpen: (prefsOpen) => set({ prefsOpen }),
  setActiveSheet: (activeSheet) => set({ activeSheet }),
}));
