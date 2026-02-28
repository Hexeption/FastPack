import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { useStore } from "../store";

interface PublishFinishedPayload {
	file_count: number;
	directory: string;
	log: import("../types").LogEntry[];
}

interface PublishFailedPayload {
	error: string;
}

function now(): string {
	const d = new Date();
	const pad = (n: number) => String(n).padStart(2, "0");
	return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

export function usePublish() {
	const setIsPublishing = useStore((s) => s.setIsPublishing);
	const appendLog = useStore((s) => s.appendLog);
	const setLog = useStore((s) => s.setLog);

	useEffect(() => {
		const unlisteners = Promise.all([
			listen("publish:started", () => {
				setIsPublishing(true);
				appendLog({ level: "info", message: "Publishing…", time: now() });
			}),

			listen<PublishFinishedPayload>("publish:finished", ({ payload }) => {
				setIsPublishing(false);
				setLog(payload.log);
			}),

			listen<PublishFailedPayload>("publish:failed", ({ payload }) => {
				setIsPublishing(false);
				appendLog({ level: "error", message: payload.error, time: now() });
			}),
		]);

		return () => {
			unlisteners.then((fns) => fns.forEach((fn) => fn()));
		};
	}, [setIsPublishing, appendLog, setLog]);
}
