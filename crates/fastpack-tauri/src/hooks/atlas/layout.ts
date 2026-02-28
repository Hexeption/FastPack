import type { SheetData } from "../../types";
import { SHEET_GAP } from "./constants";

/** Computes the total width (with gaps) and max height across all sheets. */
export function getLayout(sheets: SheetData[]) {
	if (sheets.length === 0) return { totalW: 0, maxH: 0 };
	const totalW =
		sheets.reduce((s, sh) => s + sh.width, 0) + SHEET_GAP * (sheets.length - 1);
	const maxH = Math.max(...sheets.map((sh) => sh.height));
	return { totalW, maxH };
}
