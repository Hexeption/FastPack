import { useCallback, useRef, useState } from "react";
import type { SheetData } from "../../types";
import { hitTest as hitTestImpl, marqueeHitTest } from "./hitTest";
import { getLayout } from "./layout";

function clampPan(
	pan: { x: number; y: number },
	zoom: number,
	totalW: number,
	maxH: number,
	w: number,
	h: number,
): { x: number; y: number } {
	const contentW = totalW * zoom;
	const contentH = maxH * zoom;
	const maxPanX = Math.max(0, (contentW - w) / 2) + w * 0.3;
	const maxPanY = Math.max(0, (contentH - h) / 2) + h * 0.3;
	return {
		x: Math.max(-maxPanX, Math.min(maxPanX, pan.x)),
		y: Math.max(-maxPanY, Math.min(maxPanY, pan.y)),
	};
}

export { clampPan };

type Mode = "idle" | "pan" | "click-pending" | "marquee";

export interface InteractionRefs {
	sheetsRef: React.RefObject<SheetData[]>;
	zoomRef: React.RefObject<number>;
	panRef: React.MutableRefObject<{ x: number; y: number }>;
	sizeRef: React.RefObject<{ w: number; h: number }>;
	containerRef: React.RefObject<HTMLDivElement | null>;
	onCanvasClickRef: React.RefObject<
		((hit: string | null, ctrlKey: boolean) => void) | undefined
	>;
	onMarqueeSelectRef: React.RefObject<
		((frameIds: string[], ctrlKey: boolean) => void) | undefined
	>;
	scheduleDraw: () => void;
}

export function useInteraction(refs: InteractionRefs) {
	const {
		sheetsRef,
		zoomRef,
		panRef,
		sizeRef,
		containerRef,
		onCanvasClickRef,
		onMarqueeSelectRef,
		scheduleDraw,
	} = refs;

	const [isOverFrame, setIsOverFrame] = useState(false);
	const modeRef = useRef<Mode>("idle");
	const panDragRef = useRef<{
		mx: number;
		my: number;
		px: number;
		py: number;
	} | null>(null);
	const marqueeOriginRef = useRef<{
		cx: number;
		cy: number;
		lx: number;
		ly: number;
		ctrlKey: boolean;
	} | null>(null);
	const [interactionState, setInteractionState] = useState<
		"idle" | "panning" | "marquee"
	>("idle");
	const [marquee, setMarquee] = useState<{
		x: number;
		y: number;
		w: number;
		h: number;
	} | null>(null);

	const hitTest = useCallback(
		(clientX: number, clientY: number): string | null => {
			return hitTestImpl(
				clientX,
				clientY,
				containerRef.current,
				sheetsRef.current,
				zoomRef.current,
				panRef.current,
				sizeRef.current,
			);
		},
		[containerRef, sheetsRef, zoomRef, panRef, sizeRef],
	);

	const onMouseDown = useCallback(
		(e: React.MouseEvent) => {
			if (modeRef.current !== "idle") return;
			if (e.button === 1) {
				e.preventDefault();
				modeRef.current = "pan";
				panDragRef.current = {
					mx: e.clientX,
					my: e.clientY,
					px: panRef.current.x,
					py: panRef.current.y,
				};
				setInteractionState("panning");
			} else if (e.button === 0) {
				modeRef.current = "click-pending";
				const container = containerRef.current;
				if (container) {
					const rect = container.getBoundingClientRect();
					marqueeOriginRef.current = {
						cx: e.clientX,
						cy: e.clientY,
						lx: e.clientX - rect.left,
						ly: e.clientY - rect.top,
						ctrlKey: e.ctrlKey || e.metaKey,
					};
				}
			}
		},
		[containerRef, panRef],
	);

	const onMouseMove = useCallback(
		(e: React.MouseEvent) => {
			const mode = modeRef.current;
			if (mode === "pan" && panDragRef.current) {
				const ds = panDragRef.current;
				const currentSheets = sheetsRef.current;
				const newPan = {
					x: ds.px + (e.clientX - ds.mx),
					y: ds.py + (e.clientY - ds.my),
				};
				if (currentSheets.length > 0) {
					const { totalW, maxH } = getLayout(currentSheets);
					const { w, h } = sizeRef.current;
					panRef.current = clampPan(
						newPan,
						zoomRef.current,
						totalW,
						maxH,
						w,
						h,
					);
				} else {
					panRef.current = newPan;
				}
				scheduleDraw();
			} else if (
				(mode === "click-pending" || mode === "marquee") &&
				marqueeOriginRef.current
			) {
				const origin = marqueeOriginRef.current;
				const dx = e.clientX - origin.cx;
				const dy = e.clientY - origin.cy;
				if (
					mode === "click-pending" &&
					(Math.abs(dx) > 4 || Math.abs(dy) > 4)
				) {
					modeRef.current = "marquee";
					setInteractionState("marquee");
				}
				if (modeRef.current === "marquee") {
					const container = containerRef.current;
					if (container) {
						const rect = container.getBoundingClientRect();
						const ex = e.clientX - rect.left;
						const ey = e.clientY - rect.top;
						const selRect = {
							x: Math.min(origin.lx, ex),
							y: Math.min(origin.ly, ey),
							w: Math.abs(ex - origin.lx),
							h: Math.abs(ey - origin.ly),
						};
						setMarquee(selRect);
						const ids = marqueeHitTest(
							selRect,
							sheetsRef.current,
							zoomRef.current,
							panRef.current,
							sizeRef.current,
						);
						onMarqueeSelectRef.current?.(ids, origin.ctrlKey);
					}
				}
			} else {
				setIsOverFrame(hitTest(e.clientX, e.clientY) !== null);
			}
		},
		[
			sheetsRef,
			zoomRef,
			panRef,
			sizeRef,
			containerRef,
			onMarqueeSelectRef,
			scheduleDraw,
			hitTest,
		],
	);

	const onMouseUp = useCallback(
		(e: React.MouseEvent) => {
			const mode = modeRef.current;
			if (mode === "pan") {
				panDragRef.current = null;
			} else if (mode === "click-pending") {
				const hit = hitTest(e.clientX, e.clientY);
				onCanvasClickRef.current?.(hit, e.ctrlKey || e.metaKey);
			} else if (mode === "marquee") {
				setMarquee(null);
			}
			marqueeOriginRef.current = null;
			modeRef.current = "idle";
			setInteractionState("idle");
			setIsOverFrame(hitTest(e.clientX, e.clientY) !== null);
		},
		[hitTest, onCanvasClickRef],
	);

	const onMouseLeave = useCallback(() => {
		panDragRef.current = null;
		marqueeOriginRef.current = null;
		modeRef.current = "idle";
		setInteractionState("idle");
		setIsOverFrame(false);
		setMarquee(null);
	}, []);

	const cursor =
		interactionState === "panning"
			? "grabbing"
			: interactionState === "marquee"
				? "crosshair"
				: isOverFrame
					? "pointer"
					: "default";

	return {
		cursor,
		marquee,
		hitTest,
		onMouseDown,
		onMouseMove,
		onMouseUp,
		onMouseLeave,
	};
}
