import { RotateCcw } from "lucide-react";
import { useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { useAtlasCanvas } from "../hooks/useAtlasCanvas";
import { useStore } from "../store";
import type { SheetData } from "../types";
import PanelHeader from "./PanelHeader";

function expandWithAliases(ids: string[], sheets: SheetData[]): string[] {
	const allFrames = sheets.flatMap((s) => s.frames);
	const result = new Set<string>();
	for (const id of ids) {
		const frame = allFrames.find((f) => f.id === id);
		const canonicalId = frame?.alias_of ?? id;
		result.add(canonicalId);
		for (const f of allFrames) {
			if (f.alias_of === canonicalId) result.add(f.id);
		}
	}
	return [...result];
}

export default function AtlasPreview() {
	const { t } = useTranslation();
	const sheets = useStore((s) => s.sheets);
	const isPacking = useStore((s) => s.isPacking);
	const spriteCount = useStore((s) => s.spriteCount);
	const aliasCount = useStore((s) => s.aliasCount);
	const overflowCount = useStore((s) => s.overflowCount);
	const selectedFrames = useStore((s) => s.selectedFrames);
	const setSelectedFrames = useStore((s) => s.setSelectedFrames);
	const setAnchorFrame = useStore((s) => s.setAnchorFrame);
	const zoomToFrameId = useStore((s) => s.zoomToFrameId);
	const setZoomToFrameId = useStore((s) => s.setZoomToFrameId);
	const isDark = useStore((s) => s.prefs.dark_mode);

	const onCanvasClick = useCallback(
		(hit: string | null, ctrlKey: boolean) => {
			if (hit) {
				const related = expandWithAliases([hit], sheets);
				if (ctrlKey) {
					const allSelected = related.every((id) =>
						selectedFrames.includes(id),
					);
					const next = allSelected
						? selectedFrames.filter((id) => !related.includes(id))
						: [...new Set([...selectedFrames, ...related])];
					setSelectedFrames(next);
					if (!allSelected) setAnchorFrame(hit);
				} else {
					setSelectedFrames(related);
					setAnchorFrame(hit);
				}
			} else {
				setSelectedFrames([]);
				setAnchorFrame(null);
			}
		},
		[sheets, selectedFrames, setSelectedFrames, setAnchorFrame],
	);

	const marqueeBaselineRef = useRef<string[] | null>(null);

	const onMarqueeSelect = useCallback(
		(frameIds: string[], ctrlKey: boolean) => {
			const expanded = expandWithAliases(frameIds, sheets);
			if (marqueeBaselineRef.current === null) {
				marqueeBaselineRef.current = selectedFrames;
			}
			const baseline = marqueeBaselineRef.current;
			if (ctrlKey) {
				const baseSet = new Set(baseline);
				for (const id of expanded) {
					if (baseSet.has(id)) baseSet.delete(id);
					else baseSet.add(id);
				}
				setSelectedFrames([...baseSet]);
			} else {
				setSelectedFrames(expanded);
			}
			if (expanded.length > 0) setAnchorFrame(expanded[0]);
		},
		[sheets, selectedFrames, setSelectedFrames, setAnchorFrame],
	);

	const {
		containerRef,
		zoom,
		cursor,
		marquee,
		fitView,
		zoomToFrame,
		hitTest,
		onMouseDown,
		onMouseMove,
		onMouseUp,
		onMouseLeave,
	} = useAtlasCanvas({
		sheets,
		selectedFrames,
		spriteCount,
		aliasCount,
		overflowCount,
		isDark,
		onCanvasClick,
		onMarqueeSelect,
	});

	// Reset marquee baseline when drag ends
	useEffect(() => {
		if (!marquee) marqueeBaselineRef.current = null;
	}, [marquee]);

	// React to zoomToFrameId set from SpriteList double-click
	useEffect(() => {
		if (!zoomToFrameId) return;
		zoomToFrame(zoomToFrameId);
		setZoomToFrameId(null);
	}, [zoomToFrameId, zoomToFrame, setZoomToFrameId]);

	// Auto-fit after pack completes
	const prevSheetsRef = useRef(sheets);
	useEffect(() => {
		if (sheets !== prevSheetsRef.current && sheets.length > 0) {
			fitView();
		}
		prevSheetsRef.current = sheets;
	}, [sheets, fitView]);

	return (
		<div className="flex flex-col w-full h-full overflow-hidden">
			<PanelHeader title={t("atlas.title")} className="bg-card">
				<div className="flex items-center gap-1">
					<span className="text-xs text-muted-foreground/60 tabular-nums w-10 text-right">
						{Math.round(zoom * 100)}%
					</span>
					<Button
						variant="ghost"
						size="icon-xs"
						onClick={fitView}
						title={t("atlas.fitToScreen")}
					>
						<RotateCcw className="size-3.5" />
					</Button>
				</div>
			</PanelHeader>
			<div
				ref={containerRef}
				className="relative flex-1 overflow-hidden"
				style={{ cursor }}
				onMouseDown={onMouseDown}
				onMouseMove={onMouseMove}
				onMouseUp={onMouseUp}
				onMouseLeave={onMouseLeave}
				onDoubleClick={(e) => {
					const hit = hitTest(e.clientX, e.clientY);
					if (hit) {
						zoomToFrame(hit);
					} else {
						fitView();
					}
				}}
			>
				{sheets.length === 0 && !isPacking && (
					<div className="absolute inset-0 flex items-center justify-center pointer-events-none">
						<span className="text-sm text-foreground/40">
							{t("atlas.addSprites")}
						</span>
					</div>
				)}
				{marquee && (
					<div
						className="absolute pointer-events-none border border-primary bg-primary/10"
						style={{
							left: marquee.x,
							top: marquee.y,
							width: marquee.w,
							height: marquee.h,
						}}
					/>
				)}
			</div>
		</div>
	);
}
