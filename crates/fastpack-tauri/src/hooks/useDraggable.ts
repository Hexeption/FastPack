import { useRef, useState } from "react";

export function useDraggable(initial = { x: 40, y: 80 }) {
	const [pos, setPos] = useState(initial);
	const dragRef = useRef<{
		startX: number;
		startY: number;
		ox: number;
		oy: number;
	} | null>(null);

	const onDragStart = (e: React.MouseEvent) => {
		e.preventDefault();
		dragRef.current = {
			startX: e.clientX,
			startY: e.clientY,
			ox: pos.x,
			oy: pos.y,
		};
		const onMove = (ev: MouseEvent) => {
			if (!dragRef.current) return;
			setPos({
				x: dragRef.current.ox + ev.clientX - dragRef.current.startX,
				y: dragRef.current.oy + ev.clientY - dragRef.current.startY,
			});
		};
		const onUp = () => {
			dragRef.current = null;
			window.removeEventListener("mousemove", onMove);
			window.removeEventListener("mouseup", onUp);
		};
		window.addEventListener("mousemove", onMove);
		window.addEventListener("mouseup", onUp);
	};

	return { pos, onDragStart };
}
