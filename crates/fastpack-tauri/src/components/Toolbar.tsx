import {
	Eye,
	Moon,
	PanelBottom,
	PanelLeft,
	PanelRight,
	Sun,
	Upload,
	Zap,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import {
	pack,
	publish,
	savePreferences,
	startWatch,
	stopWatch,
} from "../lib/commands";
import { useStore } from "../store";
import IconButton from "./IconButton";

export default function Toolbar() {
	const { t } = useTranslation();
	const isPacking = useStore((s) => s.isPacking);
	const isPublishing = useStore((s) => s.isPublishing);
	const isWatching = useStore((s) => s.isWatching);
	const project = useStore((s) => s.project);
	const setIsWatching = useStore((s) => s.setIsWatching);
	const showSprites = useStore((s) => s.showSprites);
	const showSettings = useStore((s) => s.showSettings);
	const showOutput = useStore((s) => s.showOutput);
	const setShowSprites = useStore((s) => s.setShowSprites);
	const setShowSettings = useStore((s) => s.setShowSettings);
	const setShowOutput = useStore((s) => s.setShowOutput);
	const prefs = useStore((s) => s.prefs);
	const setPrefs = useStore((s) => s.setPrefs);

	const hasSources = (project?.sources.length ?? 0) > 0;
	const busy = isPacking || isPublishing;

	const handlePack = () => {
		pack().catch(console.error);
	};

	const handlePublish = () => {
		publish().catch(console.error);
	};

	const handleWatch = async () => {
		if (isWatching) {
			await stopWatch();
			setIsWatching(false);
		} else {
			await startWatch();
			setIsWatching(true);
		}
	};

	return (
		<div className="flex items-center h-9 bg-card border-b border-border px-3 gap-2 shrink-0">
			<Tooltip>
				<TooltipTrigger asChild>
					<Button onClick={handlePack} disabled={busy || !hasSources} size="xs">
						<Zap className="size-3" />
						{isPacking ? t("toolbar.packing") : t("toolbar.pack")}
					</Button>
				</TooltipTrigger>
				<TooltipContent>{t("toolbar.packTooltip")}</TooltipContent>
			</Tooltip>

			<Tooltip>
				<TooltipTrigger asChild>
					<Button
						variant="outline"
						onClick={handlePublish}
						disabled={busy || !hasSources}
						size="xs"
					>
						<Upload className="size-3" />
						{isPublishing ? t("toolbar.publishing") : t("toolbar.publish")}
					</Button>
				</TooltipTrigger>
				<TooltipContent>{t("toolbar.publishTooltip")}</TooltipContent>
			</Tooltip>

			<Separator orientation="vertical" className="h-4" />

			<Tooltip>
				<TooltipTrigger asChild>
					<Button
						variant="outline"
						onClick={handleWatch}
						disabled={!hasSources}
						size="xs"
						className={cn(
							isWatching && "border-primary text-primary bg-primary/10",
						)}
					>
						{isWatching && (
							<span className="size-1.5 rounded-full bg-green-500 animate-pulse" />
						)}
						<Eye className="size-3" />
						{isWatching ? t("toolbar.watching") : t("toolbar.watch")}
					</Button>
				</TooltipTrigger>
				<TooltipContent>
					{isWatching
						? t("toolbar.stopWatchTooltip")
						: t("toolbar.watchTooltip")}
				</TooltipContent>
			</Tooltip>

			<div className="flex items-center gap-0.5 ml-auto">
				<IconButton
					icon={<PanelLeft className="size-3.5" />}
					tooltip={
						showSprites ? t("toolbar.hideSprites") : t("toolbar.showSprites")
					}
					size="icon-xs"
					onClick={() => setShowSprites(!showSprites)}
					className={cn(!showSprites && "opacity-40")}
				/>
				<IconButton
					icon={<PanelBottom className="size-3.5" />}
					tooltip={
						showOutput ? t("toolbar.hideOutput") : t("toolbar.showOutput")
					}
					size="icon-xs"
					onClick={() => setShowOutput(!showOutput)}
					className={cn(!showOutput && "opacity-40")}
				/>
				<IconButton
					icon={<PanelRight className="size-3.5" />}
					tooltip={
						showSettings ? t("toolbar.hideSettings") : t("toolbar.showSettings")
					}
					size="icon-xs"
					onClick={() => setShowSettings(!showSettings)}
					className={cn(!showSettings && "opacity-40")}
				/>
				<Separator orientation="vertical" className="h-3.5 mx-1" />
				<IconButton
					icon={
						prefs.dark_mode ? (
							<Sun className="size-3.5" />
						) : (
							<Moon className="size-3.5" />
						)
					}
					tooltip={prefs.dark_mode ? t("menu.lightTheme") : t("menu.darkTheme")}
					size="icon-xs"
					onClick={() => {
						const next = { ...prefs, dark_mode: !prefs.dark_mode };
						setPrefs(next);
						savePreferences(next).catch(console.error);
					}}
				/>
			</div>
		</div>
	);
}
