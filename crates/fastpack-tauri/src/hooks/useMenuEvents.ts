import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import {
	newProject,
	openFileDialog,
	openProject,
	saveFileDialog,
	savePreferences,
	saveProject,
} from "../lib/commands";
import { useStore } from "../store";

const EMPTY_SHEETS = {
	sheets: [],
	log: [],
	spriteCount: 0,
	aliasCount: 0,
	overflowCount: 0,
};

function resetSelection() {
	const s = useStore.getState();
	s.setSelectedFrames([]);
	s.setAnchorFrame(null);
	s.setAnimPreviewOpen(false);
}

export function useMenuEvents() {
	useEffect(() => {
		const unlisteners = Promise.all([
			listen("menu:new-project", async () => {
				const p = await newProject();
				const s = useStore.getState();
				s.setProject(p);
				s.setProjectPath(null);
				s.setSheets(EMPTY_SHEETS);
				resetSelection();
			}),

			listen("menu:open-project", async () => {
				const path = await openFileDialog();
				if (!path) return;
				try {
					const p = await openProject(path);
					const s = useStore.getState();
					s.setProject(p);
					s.setProjectPath(path);
					s.setSheets(EMPTY_SHEETS);
					resetSelection();
				} catch (e) {
					console.error(e);
				}
			}),

			listen("menu:save", async () => {
				const { project, projectPath } = useStore.getState();
				if (!project) return;
				if (projectPath) {
					await saveProject(projectPath);
					useStore.getState().setDirty(false);
				} else {
					const path = await saveFileDialog("project.fpsheet");
					if (!path) return;
					await saveProject(path);
					const s = useStore.getState();
					s.setProjectPath(path);
					s.setDirty(false);
				}
			}),

			listen("menu:save-as", async () => {
				const { project } = useStore.getState();
				if (!project) return;
				const path = await saveFileDialog("project.fpsheet");
				if (!path) return;
				await saveProject(path);
				const s = useStore.getState();
				s.setProjectPath(path);
				s.setDirty(false);
			}),

			listen("menu:toggle-theme", () => {
				const { prefs } = useStore.getState();
				const next = { ...prefs, dark_mode: !prefs.dark_mode };
				useStore.getState().setPrefs(next);
				savePreferences(next);
			}),

			listen("menu:preferences", () => {
				useStore.getState().setPrefsOpen(true);
			}),
		]);

		return () => {
			unlisteners.then((fns) => fns.forEach((fn) => fn()));
		};
	}, []);
}
