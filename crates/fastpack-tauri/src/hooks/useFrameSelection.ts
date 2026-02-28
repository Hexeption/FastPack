import { useCallback, useRef } from "react";
import { useStore } from "../store";

/** Manages frame selection with Ctrl-click (toggle), Shift-click (range), and plain click (single). Also exposes a zoom-to-frame action. */
export function useFrameSelection(visualOrder: string[]) {
	const setSelectedFrames = useStore((s) => s.setSelectedFrames);
	const setAnchorFrame = useStore((s) => s.setAnchorFrame);
	const setZoomToFrameId = useStore((s) => s.setZoomToFrameId);

	const selectedFramesRef = useRef(useStore.getState().selectedFrames);
	selectedFramesRef.current = useStore.getState().selectedFrames;
	const anchorFrameRef = useRef(useStore.getState().anchorFrame);
	anchorFrameRef.current = useStore.getState().anchorFrame;

	const visualOrderRef = useRef(visualOrder);
	visualOrderRef.current = visualOrder;

	const handleSelect = useCallback(
		(id: string, ctrlKey: boolean, shiftKey: boolean) => {
			const current = selectedFramesRef.current;
			const anchor = anchorFrameRef.current;
			const order = visualOrderRef.current;

			if (ctrlKey) {
				const next = current.includes(id)
					? current.filter((f) => f !== id)
					: [...current, id];
				setSelectedFrames(next);
				if (!current.includes(id)) setAnchorFrame(id);
			} else if (shiftKey && anchor) {
				const aIdx = order.indexOf(anchor);
				const bIdx = order.indexOf(id);
				if (aIdx === -1 || bIdx === -1) {
					setSelectedFrames([id]);
					setAnchorFrame(id);
				} else {
					const [lo, hi] = aIdx <= bIdx ? [aIdx, bIdx] : [bIdx, aIdx];
					setSelectedFrames(order.slice(lo, hi + 1));
				}
			} else {
				setSelectedFrames([id]);
				setAnchorFrame(id);
			}
		},
		[setSelectedFrames, setAnchorFrame],
	);

	const handleZoom = useCallback(
		(id: string) => {
			setSelectedFrames([id]);
			setAnchorFrame(id);
			setZoomToFrameId(id);
		},
		[setSelectedFrames, setAnchorFrame, setZoomToFrameId],
	);

	return { handleSelect, handleZoom };
}
