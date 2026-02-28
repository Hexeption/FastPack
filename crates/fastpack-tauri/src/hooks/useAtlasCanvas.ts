import * as PIXI from "pixi.js";
import { useCallback, useEffect, useRef, useState } from "react";
import type { SheetData } from "../types";
import { SHEET_GAP } from "./atlas/constants";
import { useInteraction } from "./atlas/interaction";
import { getLayout } from "./atlas/layout";
import { rebuildScene } from "./atlas/scene";
import { makeCheckerTexture } from "./atlas/textures";
import { useViewNavigation, useWheelZoom } from "./atlas/zoom";

/** Sets up a PixiJS canvas for atlas preview. Manages texture caching, scene rebuilds, zoom/pan, resize observation, and mouse interaction. Returns refs and event handlers for the container. */
export function useAtlasCanvas(opts: {
	sheets: SheetData[];
	selectedFrames: string[];
	spriteCount: number;
	aliasCount: number;
	overflowCount: number;
	isDark: boolean;
	onCanvasClick?: (hit: string | null, ctrlKey: boolean) => void;
	onMarqueeSelect?: (frameIds: string[], ctrlKey: boolean) => void;
}) {
	const {
		sheets,
		selectedFrames,
		spriteCount,
		aliasCount,
		overflowCount,
		isDark,
	} = opts;

	const zoomRef = useRef(1);
	const panRef = useRef({ x: 0, y: 0 });
	const sizeRef = useRef({ w: 0, h: 0 });
	const rafRef = useRef<number | null>(null);
	const [zoomDisplay, setZoomDisplay] = useState(1);

	const containerRef = useRef<HTMLDivElement>(null);

	const appRef = useRef<PIXI.Application | null>(null);
	const worldContainerRef = useRef<PIXI.Container | null>(null);
	const hudContainerRef = useRef<PIXI.Container | null>(null);

	const imgCache = useRef<Map<string, HTMLImageElement>>(new Map());
	const pixiTexCache = useRef<Map<string, PIXI.Texture>>(new Map());
	const checkerTexRef = useRef<PIXI.Texture | null>(null);
	const frameTexsRef = useRef<PIXI.Texture[]>([]);

	const hudStatusRef = useRef<PIXI.Text | null>(null);
	const hudLabelsRef = useRef<PIXI.Text[]>([]);

	const lastSheetsRef = useRef<SheetData[] | null>(null);
	const lastSelectedRef = useRef<string[] | null>(null);
	const lastIsDarkRef = useRef<boolean | null>(null);

	const sheetsRef = useRef(sheets);
	sheetsRef.current = sheets;
	const selectedFramesRef = useRef(selectedFrames);
	selectedFramesRef.current = selectedFrames;
	const propsRef = useRef({ spriteCount, aliasCount, overflowCount, isDark });
	propsRef.current = { spriteCount, aliasCount, overflowCount, isDark };
	const onCanvasClickRef = useRef(opts.onCanvasClick);
	onCanvasClickRef.current = opts.onCanvasClick;
	const onMarqueeSelectRef = useRef(opts.onMarqueeSelect);
	onMarqueeSelectRef.current = opts.onMarqueeSelect;

	const getPixiTex = useCallback((b64: string): PIXI.Texture | null => {
		const img = imgCache.current.get(b64);
		if (!img?.complete || img.naturalWidth === 0) return null;
		if (!pixiTexCache.current.has(b64)) {
			pixiTexCache.current.set(
				b64,
				new PIXI.Texture({ source: new PIXI.ImageSource({ resource: img }) }),
			);
		}
		return pixiTexCache.current.get(b64)!;
	}, []);

	const ensureCheckerTex = useCallback((dark: boolean): PIXI.Texture => {
		if (lastIsDarkRef.current !== dark) {
			checkerTexRef.current?.destroy(true);
			checkerTexRef.current = makeCheckerTexture(dark);
			lastIsDarkRef.current = dark;
		}
		return checkerTexRef.current!;
	}, []);

	const draw = useCallback(() => {
		const app = appRef.current;
		const worldContainer = worldContainerRef.current;
		const hudContainer = hudContainerRef.current;
		if (!app || !worldContainer || !hudContainer) return;

		const { w, h } = sizeRef.current;
		if (w === 0 || h === 0) return;

		const zoom = zoomRef.current;
		const pan = panRef.current;
		const currentSheets = sheetsRef.current;
		const currentSelected = selectedFramesRef.current;
		const { spriteCount, aliasCount, overflowCount, isDark } = propsRef.current;

		if (app.renderer.width !== w || app.renderer.height !== h) {
			app.renderer.resize(w, h);
		}

		app.renderer.background.color = isDark ? 0x1e1e1e : 0xf0f0f0;

		const needsRebuild =
			lastSheetsRef.current !== currentSheets ||
			lastSelectedRef.current !== currentSelected ||
			lastIsDarkRef.current !== isDark;
		if (needsRebuild) {
			const result = rebuildScene(
				worldContainer,
				currentSheets,
				currentSelected,
				isDark,
				{ getPixiTex, ensureCheckerTex, hudContainer: hudContainerRef.current },
				hudLabelsRef.current,
				frameTexsRef.current,
			);
			hudLabelsRef.current = result.hudLabels;
			frameTexsRef.current = result.frameTexs;
			lastSheetsRef.current = currentSheets;
			lastSelectedRef.current = currentSelected;
		}

		for (const [, tex] of pixiTexCache.current) {
			tex.source.scaleMode = zoom < 2 ? "linear" : "nearest";
		}

		if (currentSheets.length > 0) {
			const { totalW, maxH } = getLayout(currentSheets);
			worldContainer.scale.set(zoom);
			worldContainer.x = w / 2 + pan.x - (totalW * zoom) / 2;
			worldContainer.y = h / 2 + pan.y - (maxH * zoom) / 2;
		}

		if (currentSheets.length > 1) {
			const { maxH } = getLayout(currentSheets);
			let ox = 0;
			for (let i = 0; i < currentSheets.length; i++) {
				const sheet = currentSheets[i];
				const label = hudLabelsRef.current[i];
				if (label) {
					const sheetLocalBottom = new PIXI.Point(
						ox + 4,
						(maxH - sheet.height) / 2 + sheet.height,
					);
					const screenPos = worldContainer.toGlobal(sheetLocalBottom);
					label.x = screenPos.x;
					label.y = screenPos.y - label.height - 4;
				}
				ox += sheet.width + SHEET_GAP;
			}
		}

		if (!hudStatusRef.current) {
			const t = new PIXI.Text({
				text: "",
				style: { fontSize: 11, fontFamily: "-apple-system, sans-serif" },
			});
			hudContainer.addChild(t);
			hudStatusRef.current = t;
		}
		const status = hudStatusRef.current;
		const textFill = isDark ? "rgba(255,255,255,0.55)" : "rgba(0,0,0,0.55)";
		if (currentSheets.length > 0) {
			const zp = Math.round(zoom * 100);
			status.text =
				currentSheets.length === 1
					? `${currentSheets[0].width}\u00d7${currentSheets[0].height}   ${spriteCount} sprites   ${aliasCount} aliases   ${overflowCount} overflow   ${zp}%`
					: `${currentSheets.length} sheets   ${spriteCount} sprites   ${aliasCount} aliases   ${overflowCount} overflow   ${zp}%`;
			status.style.fill = textFill;
			status.x = 6;
			status.y = h - 6 - status.height;
			status.visible = true;
		} else {
			status.visible = false;
		}

		app.renderer.render(app.stage);
	}, [ensureCheckerTex, getPixiTex]); // eslint-disable-line react-hooks/exhaustive-deps

	const scheduleDraw = useCallback(() => {
		if (rafRef.current !== null) return;
		rafRef.current = requestAnimationFrame(() => {
			rafRef.current = null;
			setZoomDisplay(zoomRef.current);
			draw();
		});
	}, [draw]);

	// PixiJS app init/destroy
	useEffect(() => {
		const el = containerRef.current;
		if (!el) return;
		let alive = true;

		const app = new PIXI.Application();
		app
			.init({
				backgroundAlpha: 1,
				antialias: false,
				autoDensity: true,
				resolution: window.devicePixelRatio || 1,
				width: Math.max(1, el.clientWidth),
				height: Math.max(1, el.clientHeight),
			})
			.then(() => {
				if (!alive) {
					app.destroy();
					return;
				}
				app.ticker.stop();
				const c = app.canvas as HTMLCanvasElement;
				c.style.cssText = "position:absolute;inset:0;width:100%;height:100%";
				el.appendChild(c);
				appRef.current = app;

				const worldContainer = new PIXI.Container();
				const hudContainer = new PIXI.Container();
				app.stage.addChild(worldContainer, hudContainer);
				worldContainerRef.current = worldContainer;
				hudContainerRef.current = hudContainer;

				scheduleDraw();
			});

		return () => {
			alive = false;
			frameTexsRef.current.forEach((t) => t.destroy(false));
			frameTexsRef.current = [];
			pixiTexCache.current.forEach((t) => t.destroy(false));
			pixiTexCache.current.clear();
			checkerTexRef.current?.destroy(true);
			checkerTexRef.current = null;
			appRef.current?.destroy(true, { children: true });
			appRef.current = null;
			worldContainerRef.current = null;
			hudContainerRef.current = null;
			hudStatusRef.current = null;
			hudLabelsRef.current = [];
			lastSheetsRef.current = null;
			lastSelectedRef.current = null;
			lastIsDarkRef.current = null;
		};
	}, [scheduleDraw]); // eslint-disable-line react-hooks/exhaustive-deps

	// biome-ignore lint/correctness/useExhaustiveDependencies: intentional triggers for canvas redraw
	useEffect(() => {
		scheduleDraw();
	}, [
		sheets,
		selectedFrames,
		spriteCount,
		aliasCount,
		overflowCount,
		isDark,
		scheduleDraw,
	]);

	// ResizeObserver
	useEffect(() => {
		const el = containerRef.current;
		if (!el) return;
		const ro = new ResizeObserver(([entry]) => {
			const { width, height } = entry.contentRect;
			sizeRef.current = { w: Math.round(width), h: Math.round(height) };
			scheduleDraw();
		});
		ro.observe(el);
		return () => ro.disconnect();
	}, [scheduleDraw]);

	// Image loading
	useEffect(() => {
		const current = new Set(sheets.map((s) => s.png_b64));
		for (const key of imgCache.current.keys()) {
			if (!current.has(key)) {
				imgCache.current.delete(key);
				pixiTexCache.current.get(key)?.destroy(false);
				pixiTexCache.current.delete(key);
			}
		}
		for (const sheet of sheets) {
			const b64 = sheet.png_b64;
			if (!b64 || imgCache.current.has(b64)) continue;
			const img = new Image();
			img.onload = () => {
				imgCache.current.set(b64, img);
				pixiTexCache.current.get(b64)?.destroy(false);
				pixiTexCache.current.delete(b64);
				lastSheetsRef.current = null;
				scheduleDraw();
			};
			img.src = `data:image/png;base64,${b64}`;
		}
	}, [sheets, scheduleDraw]);

	const sharedRefs = {
		sheetsRef,
		zoomRef,
		panRef,
		sizeRef,
		containerRef,
		scheduleDraw,
	};

	useWheelZoom(sharedRefs);
	const { fitView, zoomToFrame } = useViewNavigation(sharedRefs);

	const {
		cursor,
		marquee,
		hitTest,
		onMouseDown,
		onMouseMove,
		onMouseUp,
		onMouseLeave,
	} = useInteraction({
		...sharedRefs,
		onCanvasClickRef,
		onMarqueeSelectRef,
	});

	return {
		containerRef,
		zoom: zoomDisplay,
		cursor,
		marquee,
		fitView,
		zoomToFrame,
		hitTest,
		onMouseDown,
		onMouseMove,
		onMouseUp,
		onMouseLeave,
	};
}
