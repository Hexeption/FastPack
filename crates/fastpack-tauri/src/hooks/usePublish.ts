import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect } from "react";
import { now } from "../lib/time";
import { useStore } from "../store";

interface PublishFinishedPayload {
	file_count: number;
	directory: string;
	log: import("../types").LogEntry[];
}

interface PublishFailedPayload {
	error: string;
}

/** Listens for publish:started, publish:finished, and publish:failed events from the backend. Updates the publishing state and log. */
export function usePublish() {
	const setIsPublishing = useStore((s) => s.setIsPublishing);
	const appendLog = useStore((s) => s.appendLog);
	const setLog = useStore((s) => s.setLog);

	useEffect(() => {
		const win = getCurrentWebviewWindow();
		const unlisteners = Promise.all([
			win.listen("publish:started", () => {
				setIsPublishing(true);
				appendLog({ level: "info", message: "Publishing...", time: now() });
			}),

			win.listen<PublishFinishedPayload>("publish:finished", ({ payload }) => {
				setIsPublishing(false);
				setLog(payload.log);
			}),

			win.listen<PublishFailedPayload>("publish:failed", ({ payload }) => {
				setIsPublishing(false);
				appendLog({ level: "error", message: payload.error, time: now() });
			}),
		]);

		return () => {
			unlisteners.then((fns) => fns.forEach((fn) => fn()));
		};
	}, [setIsPublishing, appendLog, setLog]);
}
