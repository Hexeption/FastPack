import { create } from "zustand";
import { DEFAULT_PREFS } from "./lib/defaults";
import type { LogEntry, Preferences, Project, SheetData } from "./types";

interface SheetsPayload {
	sheets: SheetData[];
	log: LogEntry[];
	spriteCount: number;
	aliasCount: number;
	overflowCount: number;
}

interface AppStore {
	project: Project | null;
	projectPath: string | null;
	sheets: SheetData[];
	log: LogEntry[];
	isPacking: boolean;
	isWatching: boolean;
	dirty: boolean;
	prefs: Preferences;
	prefsOpen: boolean;
	activeSheet: number;
	spriteCount: number;
	aliasCount: number;
	overflowCount: number;
	selectedFrames: string[];
	anchorFrame: string | null;
	animPreviewOpen: boolean;
	zoomToFrameId: string | null;
	showSprites: boolean;
	showSettings: boolean;
	showOutput: boolean;

	setProject: (p: Project) => void;
	setProjectPath: (path: string | null) => void;
	setSheets: (data: SheetsPayload) => void;
	appendLog: (entry: LogEntry) => void;
	setLog: (entries: LogEntry[]) => void;
	setIsPacking: (v: boolean) => void;
	setIsWatching: (v: boolean) => void;
	setDirty: (v: boolean) => void;
	setPrefs: (p: Preferences) => void;
	setPrefsOpen: (v: boolean) => void;
	setActiveSheet: (i: number) => void;
	setSelectedFrames: (ids: string[]) => void;
	setAnchorFrame: (id: string | null) => void;
	setAnimPreviewOpen: (v: boolean) => void;
	setZoomToFrameId: (id: string | null) => void;
	setShowSprites: (v: boolean) => void;
	setShowSettings: (v: boolean) => void;
	setShowOutput: (v: boolean) => void;
}

export const useStore = create<AppStore>((set) => ({
	project: null,
	projectPath: null,
	sheets: [],
	log: [],
	isPacking: false,
	isWatching: false,
	dirty: false,
	prefs: DEFAULT_PREFS,
	prefsOpen: false,
	activeSheet: 0,
	spriteCount: 0,
	aliasCount: 0,
	overflowCount: 0,
	selectedFrames: [],
	anchorFrame: null,
	animPreviewOpen: false,
	zoomToFrameId: null,
	showSprites: true,
	showSettings: true,
	showOutput: true,

	setProject: (p) => set({ project: p, dirty: false }),
	setProjectPath: (projectPath) => set({ projectPath }),
	setSheets: ({ sheets, log, spriteCount, aliasCount, overflowCount }) =>
		set({
			sheets,
			log,
			activeSheet: 0,
			spriteCount,
			aliasCount,
			overflowCount,
		}),
	appendLog: (entry) => set((s) => ({ log: [...s.log, entry] })),
	setLog: (log) => set({ log }),
	setIsPacking: (isPacking) => set({ isPacking }),
	setIsWatching: (isWatching) => set({ isWatching }),
	setDirty: (dirty) => set({ dirty }),
	setPrefs: (prefs) => set({ prefs }),
	setPrefsOpen: (prefsOpen) => set({ prefsOpen }),
	setActiveSheet: (activeSheet) => set({ activeSheet }),
	setSelectedFrames: (selectedFrames) => set({ selectedFrames }),
	setAnchorFrame: (anchorFrame) => set({ anchorFrame }),
	setAnimPreviewOpen: (animPreviewOpen) => set({ animPreviewOpen }),
	setZoomToFrameId: (zoomToFrameId) => set({ zoomToFrameId }),
	setShowSprites: (showSprites) => set({ showSprites }),
	setShowSettings: (showSettings) => set({ showSettings }),
	setShowOutput: (showOutput) => set({ showOutput }),
}));
