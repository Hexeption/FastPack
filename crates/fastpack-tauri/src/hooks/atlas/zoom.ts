import { useCallback, useEffect, useRef } from "react";
import { useStore } from "../../store";
import type { SheetData } from "../../types";
import {
	SHEET_GAP,
	ZOOM_IN_FACTOR,
	ZOOM_MAX,
	ZOOM_MIN,
	ZOOM_OUT_FACTOR,
} from "./constants";
import { clampPan } from "./interaction";
import { getLayout } from "./layout";

export interface ZoomRefs {
	sheetsRef: React.RefObject<SheetData[]>;
	zoomRef: React.MutableRefObject<number>;
	panRef: React.MutableRefObject<{ x: number; y: number }>;
	sizeRef: React.RefObject<{ w: number; h: number }>;
	containerRef: React.RefObject<HTMLDivElement | null>;
	scheduleDraw: () => void;
}

export function useWheelZoom(refs: ZoomRefs) {
	const { sheetsRef, zoomRef, panRef, sizeRef, containerRef, scheduleDraw } =
		refs;
	const zoomSpeed = useStore((s) => s.prefs.atlas_zoom_speed ?? 1);
	const invertScroll = useStore((s) => s.prefs.atlas_invert_scroll ?? false);
	const zoomPrefsRef = useRef({ zoomSpeed, invertScroll });
	zoomPrefsRef.current = { zoomSpeed, invertScroll };

	useEffect(() => {
		const el = containerRef.current;
		if (!el) return;
		const handleWheel = (e: WheelEvent) => {
			e.preventDefault();
			const { zoomSpeed, invertScroll } = zoomPrefsRef.current;
			const deltaY = invertScroll ? -e.deltaY : e.deltaY;
			const baseFactor = deltaY < 0 ? ZOOM_IN_FACTOR : ZOOM_OUT_FACTOR;
			const speed = Math.max(0.1, zoomSpeed);
			const factor =
				deltaY < 0
					? 1 + (baseFactor - 1) * speed
					: 1 - (1 - baseFactor) * speed;
			const oldZoom = zoomRef.current;
			const newZoom = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, oldZoom * factor));
			if (newZoom === oldZoom) return;

			const rect = el.getBoundingClientRect();
			const localX = e.clientX - rect.left;
			const localY = e.clientY - rect.top;
			const { w, h } = sizeRef.current;
			const pan = panRef.current;
			const currentSheets = sheetsRef.current;
			const { totalW, maxH } = getLayout(currentSheets);

			const worldX =
				(localX - (w / 2 + pan.x - (totalW * oldZoom) / 2)) / oldZoom;
			const worldY =
				(localY - (h / 2 + pan.y - (maxH * oldZoom) / 2)) / oldZoom;

			const unclamped = {
				x: localX - w / 2 + (totalW * newZoom) / 2 - worldX * newZoom,
				y: localY - h / 2 + (maxH * newZoom) / 2 - worldY * newZoom,
			};
			panRef.current = clampPan(unclamped, newZoom, totalW, maxH, w, h);
			zoomRef.current = newZoom;
			scheduleDraw();
		};
		el.addEventListener("wheel", handleWheel, { passive: false });
		return () => el.removeEventListener("wheel", handleWheel);
	}, [containerRef, sheetsRef, zoomRef, panRef, sizeRef, scheduleDraw]);
}

export function useViewNavigation(refs: ZoomRefs) {
	const { sheetsRef, zoomRef, panRef, sizeRef, scheduleDraw } = refs;

	const fitView = useCallback(() => {
		const currentSheets = sheetsRef.current;
		const { w, h } = sizeRef.current;
		if (currentSheets.length === 0 || w === 0 || h === 0) {
			zoomRef.current = 1;
			panRef.current = { x: 0, y: 0 };
			scheduleDraw();
			return;
		}
		const { totalW, maxH } = getLayout(currentSheets);
		const nz = Math.max(
			ZOOM_MIN,
			Math.min(ZOOM_MAX, Math.min((w * 0.92) / totalW, (h * 0.92) / maxH)),
		);
		zoomRef.current = nz;
		panRef.current = { x: 0, y: 0 };
		scheduleDraw();
	}, [sheetsRef, sizeRef, zoomRef, panRef, scheduleDraw]);

	const zoomToFrame = useCallback(
		(frameId: string) => {
			const currentSheets = sheetsRef.current;
			const { w, h } = sizeRef.current;
			if (currentSheets.length === 0 || w === 0 || h === 0) return;
			const { totalW } = getLayout(currentSheets);
			let sheetOffset = 0;
			for (const sheet of currentSheets) {
				const frame = sheet.frames.find((f) => f.id === frameId);
				if (frame) {
					const nz = Math.max(
						ZOOM_MIN,
						Math.min(
							ZOOM_MAX,
							Math.min((w * 0.65) / frame.w, (h * 0.65) / frame.h),
						),
					);
					zoomRef.current = nz;
					panRef.current = {
						x: nz * (totalW / 2 - sheetOffset - frame.x - frame.w / 2),
						y: nz * (sheet.height / 2 - frame.y - frame.h / 2),
					};
					scheduleDraw();
					return;
				}
				sheetOffset += sheet.width + SHEET_GAP;
			}
		},
		[sheetsRef, sizeRef, zoomRef, panRef, scheduleDraw],
	);

	return { fitView, zoomToFrame };
}
