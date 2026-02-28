import { useEffect, useRef } from "react";

interface Shortcut {
	key: string;
	mod?: boolean;
	shift?: boolean;
	action: () => void;
}

/** Registers global keydown listeners for a list of shortcuts. Matches key, modifier (Cmd/Ctrl), and shift. Skips non-modifier shortcuts when focused inside an input field. */
export function useKeyboardShortcuts(shortcuts: Shortcut[]) {
	const ref = useRef(shortcuts);
	ref.current = shortcuts;

	useEffect(() => {
		const onKey = (e: KeyboardEvent) => {
			const tag = (e.target as HTMLElement).tagName;
			const mod = e.metaKey || e.ctrlKey;

			for (const s of ref.current) {
				const keyMatch = e.key.toLowerCase() === s.key.toLowerCase();
				if (!keyMatch) continue;
				if (s.mod && !mod) continue;
				if (!s.mod && mod) continue;
				if (s.shift && !e.shiftKey) continue;
				if (!s.shift && e.shiftKey && s.mod) continue;
				// Skip non-mod shortcuts when in input fields
				if (!s.mod && (tag === "INPUT" || tag === "TEXTAREA")) continue;

				e.preventDefault();
				s.action();
				return;
			}
		};
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	}, []);
}
