import * as PIXI from "pixi.js";
import type { SheetData } from "../../types";
import { SHEET_GAP } from "./constants";
import { getLayout } from "./layout";

export interface SceneDeps {
	getPixiTex: (b64: string) => PIXI.Texture | null;
	ensureCheckerTex: (dark: boolean) => PIXI.Texture;
	hudContainer: PIXI.Container | null;
}

// Cached Link2 icon texture — built once from the lucide SVG paths
let _linkIconTex: PIXI.Texture | null = null;
function getLinkIconTex(size = 11): PIXI.Texture {
	if (_linkIconTex) return _linkIconTex;
	const canvas = document.createElement("canvas");
	canvas.width = size;
	canvas.height = size;
	const ctx = canvas.getContext("2d")!;
	const s = size / 24;
	ctx.save();
	ctx.scale(s, s);
	ctx.strokeStyle = "rgba(255,255,255,0.85)";
	ctx.lineWidth = 2.5;
	ctx.lineCap = "round";
	ctx.lineJoin = "round";
	// lucide Link2 paths (24×24 viewBox)
	ctx.stroke(new Path2D("M9 17H7A5 5 0 0 1 7 7h2"));
	ctx.stroke(new Path2D("M15 7h2a5 5 0 0 1 0 10h-2"));
	ctx.stroke(new Path2D("M11 12h2"));
	ctx.restore();
	_linkIconTex = PIXI.Texture.from(canvas);
	return _linkIconTex;
}

export function rebuildScene(
	worldContainer: PIXI.Container,
	currentSheets: SheetData[],
	currentSelected: string[],
	dark: boolean,
	deps: SceneDeps,
	prevHudLabels: PIXI.Text[],
	prevFrameTexs: PIXI.Texture[],
): { hudLabels: PIXI.Text[]; frameTexs: PIXI.Texture[] } {
	worldContainer
		.removeChildren()
		.forEach((c) => c.destroy({ children: true, texture: false }));
	prevHudLabels.forEach((t) => t.destroy());
	prevFrameTexs.forEach((t) => t.destroy(false));

	const hudLabels: PIXI.Text[] = [];
	const frameTexs: PIXI.Texture[] = [];

	if (currentSheets.length === 0) return { hudLabels, frameTexs };

	const checkerTex = deps.ensureCheckerTex(dark);
	const { maxH } = getLayout(currentSheets);
	const textFill = dark ? "rgba(255,255,255,0.55)" : "rgba(0,0,0,0.55)";

	let ox = 0;
	for (let i = 0; i < currentSheets.length; i++) {
		const sheet = currentSheets[i];
		const sc = new PIXI.Container();
		sc.x = ox;
		sc.y = (maxH - sheet.height) / 2;

		const checker = new PIXI.TilingSprite({
			texture: checkerTex,
			width: sheet.width,
			height: sheet.height,
		});
		sc.addChild(checker);

		const tex = deps.getPixiTex(sheet.png_b64);
		if (tex) {
			const spr = new PIXI.Sprite(tex);
			spr.width = sheet.width;
			spr.height = sheet.height;
			tex.source.scaleMode = "linear";
			sc.addChild(spr);
		}

		if (currentSelected.length > 0) {
			const sheetSel = sheet.frames.filter((f) =>
				currentSelected.includes(f.id),
			);
			const hasUnsel = sheet.frames.some(
				(f) => !currentSelected.includes(f.id),
			);
			if (sheetSel.length > 0 || hasUnsel) {
				const overlay = new PIXI.Graphics();
				overlay
					.rect(0, 0, sheet.width, sheet.height)
					.fill({ color: 0x000000, alpha: 0.55 });
				sc.addChild(overlay);

				if (tex) {
					// Deduplicate by atlas position — aliases share the same (x,y,w,h) as
					// their canonical and would otherwise draw the highlight twice.
					const seenPos = new Set<string>();
					for (const f of sheetSel) {
						const posKey = `${f.x},${f.y},${f.w},${f.h}`;
						if (seenPos.has(posKey)) continue;
						seenPos.add(posKey);

						const ft = new PIXI.Texture({
							source: tex.source,
							frame: new PIXI.Rectangle(f.x, f.y, f.w, f.h),
						});
						frameTexs.push(ft);
						const fspr = new PIXI.Sprite(ft);
						fspr.x = f.x;
						fspr.y = f.y;
						sc.addChild(fspr);

						const border = new PIXI.Graphics();
						border
							.rect(f.x + 1, f.y + 1, f.w - 2, f.h - 2)
							.stroke({ color: 0x3b82f6, width: 2, alignment: 0 });
						sc.addChild(border);
					}
				}
			}
		}

		worldContainer.addChild(sc);

		// Alias corner badges — draw after everything so they sit on top
		for (const f of sheet.frames) {
			if (!f.alias_of) continue;
			const ICON = 11;
			const bg = new PIXI.Graphics();
			bg.roundRect(f.x + 1, f.y + 1, ICON, ICON, 2).fill({
				color: 0x000000,
				alpha: 0.55,
			});
			sc.addChild(bg);
			const spr = new PIXI.Sprite(getLinkIconTex(ICON));
			spr.x = f.x + 1;
			spr.y = f.y + 1;
			sc.addChild(spr);
		}

		if (currentSheets.length > 1) {
			const label = new PIXI.Text({
				text: `${i + 1}  ${sheet.width}\u00d7${sheet.height}`,
				style: {
					fontSize: 10,
					fill: textFill,
					fontFamily: "-apple-system, sans-serif",
				},
			});
			deps.hudContainer?.addChild(label);
			hudLabels.push(label);
		}

		ox += sheet.width + SHEET_GAP;
	}

	return { hudLabels, frameTexs };
}
