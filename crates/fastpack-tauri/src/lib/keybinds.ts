/** Keybind formatting and comparison helpers. */

import type { Keybind } from "../types";

/** Formats a keybind for display. Uses platform-appropriate modifier symbols. */
export function formatKeybind(kb: Keybind): string {
	const isMac = navigator.userAgent.includes("Mac");
	let s = "";
	if (kb.modifier) s += isMac ? "⌘" : "Ctrl+";
	if (kb.shift) s += "⇧";
	s += kb.key.toUpperCase();
	return s;
}

/** Returns true if two keybinds have the same key, modifier, and shift state. */
export function keybindEqual(a: Keybind, b: Keybind): boolean {
	return (
		a.key.toLowerCase() === b.key.toLowerCase() &&
		a.modifier === b.modifier &&
		a.shift === b.shift
	);
}
