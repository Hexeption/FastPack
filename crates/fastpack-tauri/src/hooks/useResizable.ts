import { useEffect, useState } from "react";

interface UseResizableOptions {
	initialW: number;
	initialH: number;
	minW?: number;
	minH?: number;
}

/** Tracks resize dimensions via mouse drag with min width/height constraints. */
export function useResizable({
	initialW,
	initialH,
	minW = 200,
	minH = 240,
}: UseResizableOptions) {
	const [size, setSize] = useState({ w: initialW, h: initialH });

	const onResizeStart = (e: React.MouseEvent) => {
		e.preventDefault();
		e.stopPropagation();
		const startX = e.clientX;
		const startY = e.clientY;
		const startW = size.w;
		const startH = size.h;
		const onMove = (ev: MouseEvent) => {
			setSize({
				w: Math.max(minW, startW + ev.clientX - startX),
				h: Math.max(minH, startH + ev.clientY - startY),
			});
		};
		const onUp = () => {
			window.removeEventListener("mousemove", onMove);
			window.removeEventListener("mouseup", onUp);
		};
		window.addEventListener("mousemove", onMove);
		window.addEventListener("mouseup", onUp);
	};

	return { size, onResizeStart };
}

/** Zooms an element on scroll wheel. Clamps between 0.25x and 32x. */
export function useScrollZoom(
	ref: React.RefObject<HTMLDivElement | null>,
	enabled: boolean,
	setZoom: React.Dispatch<React.SetStateAction<number>>,
) {
	useEffect(() => {
		const el = ref.current;
		if (!el || !enabled) return;
		const handleWheel = (e: WheelEvent) => {
			e.preventDefault();
			setZoom((z) => {
				const factor = e.deltaY < 0 ? 1.15 : 0.87;
				return Math.max(0.25, Math.min(32, z * factor));
			});
		};
		el.addEventListener("wheel", handleWheel, { passive: false });
		return () => el.removeEventListener("wheel", handleWheel);
	}, [ref, enabled, setZoom]);
}
