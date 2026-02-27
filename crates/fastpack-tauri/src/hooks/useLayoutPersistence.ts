import { useCallback, useEffect, useRef } from "react";
import { useGroupRef, usePanelRef } from "react-resizable-panels";
import { useStore } from "../store";

const LAYOUT_V_KEY = "fp-layout-v";
const LAYOUT_H_KEY = "fp-layout-h";

function loadLayout(key: string): Record<string, number> | undefined {
	try {
		const raw = localStorage.getItem(key);
		if (raw) return JSON.parse(raw);
	} catch {
		/* ignore */
	}
	return undefined;
}

function saveLayout(key: string, layout: Record<string, number>) {
	try {
		localStorage.setItem(key, JSON.stringify(layout));
	} catch {
		/* ignore */
	}
}

export function useLayoutPersistence() {
	const showSprites = useStore((s) => s.showSprites);
	const showSettings = useStore((s) => s.showSettings);
	const showOutput = useStore((s) => s.showOutput);

	const spritesRef = usePanelRef();
	const settingsRef = usePanelRef();
	const outputRef = usePanelRef();
	const groupVRef = useGroupRef();
	const groupHRef = useGroupRef();

	const readyToSaveRef = useRef(false);

	const onLayoutChangedV = useCallback((layout: Record<string, number>) => {
		if (readyToSaveRef.current) saveLayout(LAYOUT_V_KEY, layout);
	}, []);
	const onLayoutChangedH = useCallback((layout: Record<string, number>) => {
		if (readyToSaveRef.current) saveLayout(LAYOUT_H_KEY, layout);
	}, []);

	// Restore saved layouts on mount, then enable saving
	useEffect(() => {
		const savedV = loadLayout(LAYOUT_V_KEY);
		const savedH = loadLayout(LAYOUT_H_KEY);
		if (savedV) {
			groupVRef.current?.setLayout(savedV);
			if (savedV.output === 0) useStore.getState().setShowOutput(false);
		}
		if (savedH) {
			groupHRef.current?.setLayout(savedH);
			if (savedH.sprites === 0) useStore.getState().setShowSprites(false);
			if (savedH.settings === 0) useStore.getState().setShowSettings(false);
		}
		requestAnimationFrame(() => {
			readyToSaveRef.current = true;
		});
	}, [groupVRef, groupHRef]);

	// Sync panel collapse/expand with toggle state
	useEffect(() => {
		showSprites ? spritesRef.current?.expand() : spritesRef.current?.collapse();
	}, [showSprites, spritesRef]);

	useEffect(() => {
		showSettings
			? settingsRef.current?.expand()
			: settingsRef.current?.collapse();
	}, [showSettings, settingsRef]);

	useEffect(() => {
		showOutput ? outputRef.current?.expand() : outputRef.current?.collapse();
	}, [showOutput, outputRef]);

	return {
		spritesRef,
		settingsRef,
		outputRef,
		groupVRef,
		groupHRef,
		onLayoutChangedV,
		onLayoutChangedH,
	};
}
