import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { pack } from "../lib/commands";
import { now } from "../lib/time";
import { useStore } from "../store";
import type { PackFailedPayload, PackFinishedPayload } from "../types";
import { useKeyboardShortcuts } from "./useKeyboardShortcuts";

/** Listens for pack:started, pack:finished, and pack:failed events from the backend. Updates sheets and log accordingly. Binds Cmd+P to trigger a pack. */
export function usePack() {
	const setIsPacking = useStore((s) => s.setIsPacking);
	const setSheets = useStore((s) => s.setSheets);
	const appendLog = useStore((s) => s.appendLog);

	useKeyboardShortcuts([
		{ key: "p", mod: true, action: () => pack().catch(console.error) },
	]);

	useEffect(() => {
		const unlisteners = Promise.all([
			listen("pack:started", () => {
				setIsPacking(true);
				appendLog({ level: "info", message: "Packing...", time: now() });
			}),

			listen<PackFinishedPayload>("pack:finished", ({ payload }) => {
				setIsPacking(false);
				setSheets({
					sheets: payload.sheets,
					log: payload.log,
					spriteCount: payload.sprite_count,
					aliasCount: payload.alias_count,
					overflowCount: payload.overflow_count,
				});
			}),

			listen<PackFailedPayload>("pack:failed", ({ payload }) => {
				setIsPacking(false);
				appendLog({ level: "error", message: payload.error, time: now() });
			}),
		]);

		return () => {
			unlisteners.then((fns) => fns.forEach((fn) => fn()));
		};
	}, [setIsPacking, setSheets, appendLog]);
}
