import type { SheetData } from "../../types";
import { SHEET_GAP } from "./constants";
import { getLayout } from "./layout";

/** Tests a screen-space point against all frame boundaries across all sheets. Returns the frame id under the cursor or null. */
export function hitTest(
	clientX: number,
	clientY: number,
	container: HTMLDivElement | null,
	sheets: SheetData[],
	zoom: number,
	pan: { x: number; y: number },
	size: { w: number; h: number },
): string | null {
	if (!container || sheets.length === 0) return null;
	const rect = container.getBoundingClientRect();
	const localX = clientX - rect.left;
	const localY = clientY - rect.top;
	const { w, h } = size;
	const { totalW, maxH } = getLayout(sheets);
	const cx = w / 2 + pan.x;
	const cy = h / 2 + pan.y;
	let ox = cx - (totalW * zoom) / 2;
	for (const sheet of sheets) {
		const sw = sheet.width * zoom;
		const sheetY = cy - (maxH * zoom) / 2 + ((maxH - sheet.height) * zoom) / 2;
		for (const frame of sheet.frames) {
			const fx = ox + frame.x * zoom;
			const fy = sheetY + frame.y * zoom;
			if (
				localX >= fx &&
				localX <= fx + frame.w * zoom &&
				localY >= fy &&
				localY <= fy + frame.h * zoom
			) {
				return frame.id;
			}
		}
		ox += sw + SHEET_GAP;
	}
	return null;
}

/** Returns all frame ids whose screen-space bounds overlap the given marquee rectangle. */
export function marqueeHitTest(
	selRect: { x: number; y: number; w: number; h: number },
	sheets: SheetData[],
	zoom: number,
	pan: { x: number; y: number },
	size: { w: number; h: number },
): string[] {
	if (sheets.length === 0) return [];
	const ids: string[] = [];
	const { w, h } = size;
	const { totalW, maxH } = getLayout(sheets);
	const cx = w / 2 + pan.x;
	const cy = h / 2 + pan.y;
	let ox = cx - (totalW * zoom) / 2;
	const sr = selRect.x + selRect.w;
	const sb = selRect.y + selRect.h;
	for (const sheet of sheets) {
		const sw = sheet.width * zoom;
		const sheetY = cy - (maxH * zoom) / 2 + ((maxH - sheet.height) * zoom) / 2;
		for (const frame of sheet.frames) {
			const fx = ox + frame.x * zoom;
			const fy = sheetY + frame.y * zoom;
			const fx2 = fx + frame.w * zoom;
			const fy2 = fy + frame.h * zoom;
			if (fx < sr && fx2 > selRect.x && fy < sb && fy2 > selRect.y) {
				ids.push(frame.id);
			}
		}
		ox += sw + SHEET_GAP;
	}
	return ids;
}
