import { useEffect, useState } from "react";
import { savePreferences } from "../lib/commands";
import { keybindEqual } from "../lib/keybinds";
import { useStore } from "../store";
import type { Keybind, KeybindsConfig } from "../types";

const SHORTCUT_ORDER: (keyof KeybindsConfig)[] = [
	"new_project",
	"open_project",
	"save_project",
	"save_project_as",
	"anim_preview",
];

/** Captures the next keypress to rebind a shortcut. Returns the current capture target, a setter, and a duplicate checker. Resets when the preferences dialog reopens. */
export function useKeybindCapture() {
	const prefs = useStore((s) => s.prefs);
	const setPrefs = useStore((s) => s.setPrefs);
	const prefsOpen = useStore((s) => s.prefsOpen);

	const [capturing, setCapturing] = useState<keyof KeybindsConfig | null>(null);

	useEffect(() => {
		if (prefsOpen) setCapturing(null);
	}, [prefsOpen]);

	useEffect(() => {
		if (!capturing) return;
		const onKey = (e: KeyboardEvent) => {
			const ignoredKeys = [
				"Shift",
				"Control",
				"Meta",
				"Alt",
				"CapsLock",
				"Tab",
			];
			if (ignoredKeys.includes(e.key)) return;
			e.preventDefault();
			e.stopPropagation();
			if (e.key === "Escape") {
				setCapturing(null);
				return;
			}
			const newBind: Keybind = {
				key: e.key.toLowerCase(),
				modifier: e.metaKey || e.ctrlKey,
				shift: e.shiftKey,
			};
			const next = {
				...prefs,
				keybinds: { ...prefs.keybinds, [capturing]: newBind },
			};
			setPrefs(next);
			savePreferences(next).catch(console.error);
			setCapturing(null);
		};
		window.addEventListener("keydown", onKey, { capture: true });
		return () =>
			window.removeEventListener("keydown", onKey, { capture: true });
	}, [capturing, prefs, setPrefs]);

	function hasDuplicate(id: keyof KeybindsConfig): boolean {
		const kb = prefs.keybinds[id];
		return SHORTCUT_ORDER.some(
			(other) => other !== id && keybindEqual(prefs.keybinds[other], kb),
		);
	}

	return { capturing, setCapturing, hasDuplicate };
}

export { SHORTCUT_ORDER };
