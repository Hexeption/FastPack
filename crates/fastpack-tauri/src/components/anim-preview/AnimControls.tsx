import {
	ArrowLeftRight,
	Layers,
	Pause,
	Play,
	Repeat,
	SkipBack,
	SkipForward,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import IconButton from "../IconButton";

type BgMode = "checker" | "black" | "white";
const ZOOM_PRESETS = [1, 2, 4, 8, 16] as const;

interface AnimControlsProps {
	playing: boolean;
	setPlaying: (v: boolean) => void;
	fps: number;
	setFps: (v: number) => void;
	looping: boolean;
	setLooping: (v: boolean) => void;
	pingPong: boolean;
	setPingPong: (v: boolean) => void;
	onionSkin: boolean;
	setOnionSkin: (v: boolean) => void;
	currentIdx: number;
	frameCount: number;
	stepBack: () => void;
	stepForward: () => void;
	bgMode: BgMode;
	setBgMode: (m: BgMode) => void;
	zoom: number;
	setZoom: (z: number) => void;
	directionRef: React.MutableRefObject<1 | -1>;
}

export default function AnimControls({
	playing,
	setPlaying,
	fps,
	setFps,
	looping,
	setLooping,
	pingPong,
	setPingPong,
	onionSkin,
	setOnionSkin,
	currentIdx,
	frameCount,
	stepBack,
	stepForward,
	bgMode,
	setBgMode,
	zoom,
	setZoom,
	directionRef,
}: AnimControlsProps) {
	const { t } = useTranslation();

	return (
		<div className="flex flex-col gap-1 p-1.5 border-t border-border shrink-0">
			<div className="flex items-center gap-1 justify-center">
				<Button
					variant="ghost"
					size="icon-xs"
					onClick={stepBack}
					title={t("animPreview.previousFrame")}
				>
					<SkipBack className="size-3.5" />
				</Button>
				<Button
					variant="ghost"
					size="icon-xs"
					onClick={() => setPlaying(!playing)}
					title={playing ? t("animPreview.pause") : t("animPreview.play")}
				>
					{playing ? (
						<Pause className="size-3.5" />
					) : (
						<Play className="size-3.5" />
					)}
				</Button>
				<Button
					variant="ghost"
					size="icon-xs"
					onClick={stepForward}
					title={t("animPreview.nextFrame")}
				>
					<SkipForward className="size-3.5" />
				</Button>

				<span className="text-xs text-muted-foreground tabular-nums mx-1">
					{currentIdx + 1} / {frameCount}
				</span>

				<IconButton
					icon={<Repeat className="size-3.5" />}
					tooltip={t("animPreview.loop")}
					tooltipSide="top"
					variant={looping ? "secondary" : "ghost"}
					onClick={() => setLooping(!looping)}
				/>
				<IconButton
					icon={<ArrowLeftRight className="size-3.5" />}
					tooltip={t("animPreview.pingPong")}
					tooltipSide="top"
					variant={pingPong ? "secondary" : "ghost"}
					onClick={() => {
						setPingPong(!pingPong);
						directionRef.current = 1;
					}}
				/>
				<IconButton
					icon={<Layers className="size-3.5" />}
					tooltip={t("animPreview.onionSkinning")}
					tooltipSide="top"
					variant={onionSkin ? "secondary" : "ghost"}
					onClick={() => setOnionSkin(!onionSkin)}
				/>
			</div>

			<div className="flex items-center gap-2 px-1">
				<span className="text-[10px] text-muted-foreground/60 shrink-0">
					{t("animPreview.fps")}
				</span>
				<Slider
					min={1}
					max={60}
					step={1}
					value={[fps]}
					onValueChange={([v]) => setFps(v)}
					className="flex-1"
				/>
				<span className="text-xs text-muted-foreground tabular-nums w-5 text-right">
					{fps}
				</span>
			</div>

			<div className="flex items-center gap-1.5 px-1">
				<div className="flex items-center gap-0.5">
					{(["checker", "black", "white"] as const).map((mode) => (
						<Button
							key={mode}
							variant={bgMode === mode ? "secondary" : "ghost"}
							size="icon-xs"
							onClick={() => setBgMode(mode)}
							title={t(
								`animPreview.bg${mode[0].toUpperCase() + mode.slice(1)}`,
							)}
						>
							<div
								className="size-3 rounded-sm border border-border"
								style={{
									background:
										mode === "checker"
											? "repeating-conic-gradient(#999 0% 25%, #666 0% 50%)"
											: mode === "black"
												? "#000"
												: "#fff",
									backgroundSize: mode === "checker" ? "6px 6px" : undefined,
								}}
							/>
						</Button>
					))}
				</div>

				<div className="h-3 w-px bg-border mx-0.5" />

				<div className="flex items-center gap-0.5">
					{ZOOM_PRESETS.map((z) => (
						<Button
							key={z}
							variant={zoom === z ? "secondary" : "ghost"}
							size="icon-xs"
							className="text-[10px] tabular-nums w-6"
							onClick={() => setZoom(z)}
						>
							{z}x
						</Button>
					))}
				</div>
			</div>
		</div>
	);
}
