import { convertFileSrc } from "@tauri-apps/api/core";
import { X } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { useAnimPlayback } from "../hooks/useAnimPlayback";
import { useDraggable } from "../hooks/useDraggable";
import { useResizable, useScrollZoom } from "../hooks/useResizable";
import { useStore } from "../store";
import type { AnimBg } from "../types";
import AnimControls from "./anim-preview/AnimControls";
import PanelHeader from "./PanelHeader";

export default function AnimPreview() {
	const { t } = useTranslation();
	const sheets = useStore((s) => s.sheets);
	const selectedFrames = useStore((s) => s.selectedFrames);
	const animPreviewOpen = useStore((s) => s.animPreviewOpen);
	const setAnimPreviewOpen = useStore((s) => s.setAnimPreviewOpen);
	const defaultFps = useStore((s) => s.prefs.anim_preview_fps);
	const defaultBg = useStore((s) => s.prefs.anim_preview_bg);

	const [bgMode, setBgMode] = useState<AnimBg>(defaultBg);
	const [zoom, setZoom] = useState(1);
	const viewportRef = useRef<HTMLDivElement>(null);
	const { pos, onDragStart } = useDraggable({ x: 40, y: 80 });

	const frames = useMemo(() => {
		const allFrames = sheets.flatMap((s) => s.frames);
		return selectedFrames
			.map((id) => allFrames.find((f) => f.id === id))
			.filter(Boolean) as typeof allFrames;
	}, [sheets, selectedFrames]);

	const playback = useAnimPlayback({
		frameCount: frames.length,
		isOpen: animPreviewOpen,
		initialFps: defaultFps,
	});
	const { size, onResizeStart } = useResizable({
		initialW: 320,
		initialH: 440,
	});

	// Reset zoom on open
	useEffect(() => {
		if (animPreviewOpen) setZoom(1);
	}, [animPreviewOpen]);

	useScrollZoom(viewportRef, animPreviewOpen, setZoom);

	if (!animPreviewOpen || frames.length < 2) return null;

	const frame = frames[playback.currentIdx];
	const imgSrc = frame ? convertFileSrc(frame.src_path) : null;
	const prevFrame =
		playback.currentIdx > 0 ? frames[playback.currentIdx - 1] : null;
	const nextFrame =
		playback.currentIdx < frames.length - 1
			? frames[playback.currentIdx + 1]
			: null;

	const bgStyle: React.CSSProperties =
		bgMode === "checker"
			? {
					backgroundImage:
						"repeating-conic-gradient(var(--checker-a) 0% 25%, var(--checker-b) 0% 50%)",
					backgroundSize: "16px 16px",
				}
			: { backgroundColor: bgMode === "black" ? "#000" : "#fff" };

	const imgStyle: React.CSSProperties = {
		imageRendering: "pixelated",
		transform: `scale(${zoom})`,
	};

	const onionStyle = (opacity: number): React.CSSProperties => ({
		...imgStyle,
		position: "absolute",
		opacity,
		pointerEvents: "none",
	});

	return (
		<div
			className="fixed z-50 rounded-lg border border-border bg-card shadow-2xl ring-1 ring-border/50 flex flex-col overflow-hidden"
			style={{ left: pos.x, top: pos.y, width: size.w, height: size.h }}
		>
			<PanelHeader
				title={t("animPreview.title")}
				className="cursor-move select-none"
				onMouseDown={onDragStart}
			>
				<div className="flex items-center gap-1">
					<span className="text-xs text-muted-foreground/60 tabular-nums">
						{Math.round(zoom * 100)}%
					</span>
					<Button
						variant="ghost"
						size="icon-xs"
						onClick={() => {
							playback.setPlaying(false);
							setAnimPreviewOpen(false);
						}}
					>
						<X className="size-3.5" />
					</Button>
				</div>
			</PanelHeader>

			<div
				ref={viewportRef}
				className="relative flex-1 min-h-0 flex items-center justify-center overflow-hidden"
				style={bgStyle}
			>
				{playback.onionSkin && prevFrame && (
					<img
						key={`onion-prev-${prevFrame.src_path}`}
						src={convertFileSrc(prevFrame.src_path)}
						alt=""
						className="max-w-full max-h-full object-contain"
						style={onionStyle(0.2)}
						draggable={false}
					/>
				)}
				{imgSrc && (
					<img
						key={imgSrc}
						src={imgSrc}
						alt={frame?.id}
						className="max-w-full max-h-full object-contain"
						style={imgStyle}
						draggable={false}
					/>
				)}
				{playback.onionSkin && nextFrame && (
					<img
						key={`onion-next-${nextFrame.src_path}`}
						src={convertFileSrc(nextFrame.src_path)}
						alt=""
						className="max-w-full max-h-full object-contain"
						style={onionStyle(0.2)}
						draggable={false}
					/>
				)}
				<span className="absolute bottom-1.5 left-2 text-xs text-white/50 tabular-nums">
					{frame?.id.split("/").pop()} {frame && `${frame.w}\u00d7${frame.h}`}
				</span>
			</div>

			<AnimControls
				{...playback}
				frameCount={frames.length}
				bgMode={bgMode}
				setBgMode={setBgMode}
				zoom={zoom}
				setZoom={setZoom}
			/>

			<div
				className="absolute bottom-0 right-0 w-4 h-4 cursor-se-resize opacity-40 hover:opacity-80"
				onMouseDown={onResizeStart}
			>
				<svg
					width="16"
					height="16"
					viewBox="0 0 16 16"
					className="text-muted-foreground"
				>
					<path
						d="M14 14L14 8M14 14L8 14"
						stroke="currentColor"
						strokeWidth="1.5"
						fill="none"
					/>
				</svg>
			</div>
		</div>
	);
}
