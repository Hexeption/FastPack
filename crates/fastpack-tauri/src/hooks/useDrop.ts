import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import { handleDrop, pack, startWatch, stopWatch } from "../lib/commands";
import { useStore } from "../store";

export function useDrop() {
	useEffect(() => {
		const unlisten = getCurrentWindow().onDragDropEvent(async (event) => {
			if (event.payload.type !== "drop") return;
			try {
				const result = await handleDrop(event.payload.paths);
				useStore.getState().setProject(result.project);
				useStore.getState().setProjectPath(result.project_path);
				useStore.getState().setDirty(result.dirty);
				if (result.dirty && result.project.sources.length > 0) {
					pack().catch(console.error);
					if (useStore.getState().isWatching) {
						await stopWatch();
						await startWatch();
					}
				}
			} catch (err) {
				console.error(err);
			}
		});
		return () => {
			unlisten.then((f) => f());
		};
	}, []);
}
