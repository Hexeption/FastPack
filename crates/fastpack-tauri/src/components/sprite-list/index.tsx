import { convertFileSrc } from "@tauri-apps/api/core";
import {
	Ban,
	ChevronDown,
	ChevronRight,
	Clapperboard,
	FolderPlus,
	Undo2,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
	ContextMenu,
	ContextMenuContent,
	ContextMenuItem,
	ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { Slider } from "@/components/ui/slider";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import { useFrameSelection } from "../../hooks/useFrameSelection";
import { useTreeNav } from "../../hooks/useTreeNav";
import {
	addSource,
	openFolderDialog,
	pack,
	removeSource,
	startWatch,
	stopWatch,
	updateProject,
} from "../../lib/commands";
import { buildTree, flattenTree } from "../../lib/tree";
import { useStore } from "../../store";
import IconButton from "../IconButton";
import PanelHeader from "../PanelHeader";
import { iconSize, THUMB_DEFAULT, type ThumbInfo } from "./FileRow";
import SourceRow from "./SourceRow";

export default function SpriteList() {
	const { t } = useTranslation();
	const project = useStore((s) => s.project);
	const setProject = useStore((s) => s.setProject);
	const sheets = useStore((s) => s.sheets);
	const setDirty = useStore((s) => s.setDirty);
	const setSheets = useStore((s) => s.setSheets);
	const isWatching = useStore((s) => s.isWatching);
	const selectedFrames = useStore((s) => s.selectedFrames);
	const setSelectedFrames = useStore((s) => s.setSelectedFrames);
	const setAnchorFrame = useStore((s) => s.setAnchorFrame);
	const setAnimPreviewOpen = useStore((s) => s.setAnimPreviewOpen);
	const [openSources, setOpenSources] = useState<Set<string>>(new Set());
	const [thumbSize, setThumbSize] = useState(THUMB_DEFAULT);

	const thumbMap = useMemo(() => {
		const map = new Map<string, ThumbInfo>();
		for (const sheet of sheets) {
			for (const frame of sheet.frames) {
				map.set(frame.id, {
					style: {
						width: thumbSize,
						height: thumbSize,
						backgroundImage: `url(${convertFileSrc(frame.src_path)}), repeating-conic-gradient(var(--checker-a) 0% 25%, var(--checker-b) 0% 50%)`,
						backgroundRepeat: "no-repeat, repeat",
						backgroundSize: "contain, 6px 6px",
						backgroundPosition: "center, 0 0",
					},
				});
			}
		}
		return map;
	}, [sheets, thumbSize]);

	const allFrames = useMemo(() => sheets.flatMap((s) => s.frames), [sheets]);

	const srcPathMap = useMemo(() => {
		const map = new Map<string, string>();
		for (const sheet of sheets) {
			for (const frame of sheet.frames) {
				map.set(frame.id, frame.src_path);
			}
		}
		return map;
	}, [sheets]);

	const excludedThumbMap = useMemo(() => {
		const excludes = project?.excludes ?? [];
		if (excludes.length === 0) return new Map<string, ThumbInfo>();
		const map = new Map<string, ThumbInfo>();
		for (const id of excludes) {
			const srcPath = srcPathMap.get(id);
			if (!srcPath) continue;
			map.set(id, {
				style: {
					width: thumbSize,
					height: thumbSize,
					backgroundImage: `url(${convertFileSrc(srcPath)}), repeating-conic-gradient(var(--checker-a) 0% 25%, var(--checker-b) 0% 50%)`,
					backgroundRepeat: "no-repeat, repeat",
					backgroundSize: "contain, 6px 6px",
					backgroundPosition: "center, 0 0",
				},
			});
		}
		return map;
	}, [project?.excludes, thumbSize, srcPathMap]);

	useEffect(() => {
		if (!project || allFrames.length === 0) return;
		setOpenSources((prev) => {
			const next = new Set(prev);
			project.sources.forEach((_, i) => next.add(`__src__${i}`));
			return next;
		});
	}, [allFrames.length, project]);

	const visualOrder = useMemo(() => {
		if (!project) return [];
		const order: string[] = [];
		for (let i = 0; i < project.sources.length; i++) {
			const src = project.sources[i];
			const srcName = src.path.split(/[\\/]/).pop() ?? src.path;
			const srcFrames =
				project.sources.length === 1
					? allFrames
					: allFrames.filter((f) => f.id.startsWith(`${srcName}/`));
			const tree = buildTree(srcFrames, `${srcName}/`);
			flattenTree(tree, order);
		}
		return order;
	}, [project, allFrames]);

	const { handleSelect, handleZoom } = useFrameSelection(visualOrder);

	const scrollRef = useRef<HTMLDivElement>(null);

	const { openFolders, toggleFolder, handleKeyDown, clearNav } = useTreeNav({
		scrollRef,
		project,
		allFrames,
		openSources,
		setOpenSources,
		visualOrder,
		handleSelect,
	});

	const restartWatcher = async () => {
		if (isWatching) {
			await stopWatch();
			await startWatch();
		}
	};

	const handleAddSource = async () => {
		const startingPath = project?.sources[0]?.path ?? null;
		const path = await openFolderDialog(startingPath);
		if (!path) return;
		const p = await addSource(path);
		setProject(p);
		setDirty(true);
		if (p.sources.length > 0) {
			pack().catch(console.error);
			restartWatcher();
		}
	};

	const handleRemoveSource = async (index: number) => {
		const p = await removeSource(index);
		setProject(p);
		setDirty(true);
		setSheets({
			sheets: [],
			log: [],
			spriteCount: 0,
			aliasCount: 0,
			overflowCount: 0,
		});
		setSelectedFrames([]);
		setAnchorFrame(null);
		if (p.sources.length > 0) {
			pack().catch(console.error);
			restartWatcher();
		}
	};

	const handleRestore = (id: string) => {
		if (!project) return;
		const updated = {
			...project,
			excludes: (project.excludes ?? []).filter((e) => e !== id),
		};
		setProject(updated);
		setDirty(true);
		updateProject(updated)
			.then(() => pack())
			.catch(console.error);
	};

	const handleRestoreAll = () => {
		if (!project) return;
		const updated = { ...project, excludes: [] };
		setProject(updated);
		setDirty(true);
		updateProject(updated)
			.then(() => pack())
			.catch(console.error);
	};

	const handleSetFolderColor = (key: string, color: string | null) => {
		if (!project) return;
		const next = { ...project.folder_colors };
		if (color === null) {
			delete next[key];
		} else {
			next[key] = color;
		}
		const updated = { ...project, folder_colors: next };
		setProject(updated);
		setDirty(true);
		updateProject(updated).catch(console.error);
	};

	const handleExclude = (id: string) => {
		if (!project) return;
		const toExclude =
			selectedFrames.includes(id) && selectedFrames.length > 1
				? selectedFrames
				: [id];
		const existing = new Set(project.excludes ?? []);
		for (const f of toExclude) existing.add(f);
		const updated = { ...project, excludes: [...existing] };
		setProject(updated);
		setSelectedFrames([]);
		setAnchorFrame(null);
		setDirty(true);
		updateProject(updated)
			.then(() => pack())
			.catch(console.error);
	};

	const excludes = project?.excludes ?? [];
	const [excludedOpen, setExcludedOpen] = useState(true);
	const iSize = iconSize(thumbSize);

	return (
		<div className="flex flex-col w-full h-full border-r border-border bg-card overflow-hidden">
			<PanelHeader title={t("sprites.panelTitle")}>
				<div className="flex items-center gap-0.5">
					{selectedFrames.length >= 2 && (
						<IconButton
							icon={<Clapperboard className="size-3.5" />}
							tooltip={
								<>
									{t("sprites.previewAnimation")} ({selectedFrames.length}{" "}
									frames)
								</>
							}
							tooltipSide="bottom"
							onClick={() => setAnimPreviewOpen(true)}
						/>
					)}
					<Button
						variant="ghost"
						size="icon-xs"
						onClick={handleAddSource}
						title={t("sprites.addSourceFolder")}
					>
						<FolderPlus className="size-3.5" />
					</Button>
				</div>
			</PanelHeader>
			<div
				ref={scrollRef}
				tabIndex={0}
				onKeyDown={handleKeyDown}
				className="flex-1 overflow-y-auto outline-none"
				onClickCapture={clearNav}
				onClick={() => setSelectedFrames([])}
			>
				{!project || project.sources.length === 0 ? (
					<div className="flex flex-col items-center justify-center gap-2 p-4 text-center h-full">
						<p className="text-xs text-muted-foreground">
							{t("sprites.noSources")}
						</p>
						<Button variant="outline" size="xs" onClick={handleAddSource}>
							{t("sprites.addFolder")}
						</Button>
					</div>
				) : (
					project.sources.map((src, i) => {
						const srcName = src.path.split(/[\\/]/).pop() ?? src.path;
						const srcFrames =
							project.sources.length === 1
								? allFrames
								: allFrames.filter((f) => f.id.startsWith(`${srcName}/`));
						const tree = buildTree(srcFrames, `${srcName}/`);
						const srcKey = `__src__${i}`;

						const isOpen = openSources.has(srcKey);

						return (
							<SourceRow
								key={src.path}
								srcName={srcName}
								srcPath={src.path}
								srcFrameCount={srcFrames.length}
								tree={tree}
								isOpen={isOpen}
								onToggle={() => {
									const next = new Set(openSources);
									if (isOpen) next.delete(srcKey);
									else next.add(srcKey);
									setOpenSources(next);
								}}
								navKey={srcKey}
								openFolders={openFolders}
								onToggleFolder={toggleFolder}
								onRemove={() => handleRemoveSource(i)}
								onExclude={handleExclude}
								folderColors={project.folder_colors ?? {}}
								onSetFolderColor={handleSetFolderColor}
								selectedFrames={selectedFrames}
								onSelect={handleSelect}
								onZoom={handleZoom}
								thumbMap={thumbMap}
								thumbSize={thumbSize}
								allFramesEmpty={allFrames.length === 0}
							/>
						);
					})
				)}
				{excludes.length > 0 && (
					<div>
						<div className="flex items-center justify-between px-2.5 py-1 hover:bg-accent group select-none border-b border-border/50">
							<button
								className="flex items-center gap-1.5 min-w-0 flex-1 text-left"
								onClick={(e) => {
									e.stopPropagation();
									setExcludedOpen((v) => !v);
								}}
							>
								{excludedOpen ? (
									<ChevronDown
										size={iSize}
										className="shrink-0 text-muted-foreground/50"
									/>
								) : (
									<ChevronRight
										size={iSize}
										className="shrink-0 text-muted-foreground/50"
									/>
								)}
								<Ban
									size={iSize}
									className="shrink-0 text-muted-foreground/50"
								/>
								<span className="text-xs font-medium text-muted-foreground truncate">
									{t("sprites.excluded")}
								</span>
								<span className="text-xs text-muted-foreground/60 shrink-0">
									({excludes.length})
								</span>
							</button>
							<Button
								variant="ghost"
								size="icon-xs"
								className="opacity-0 group-hover:opacity-100 shrink-0"
								onClick={(e) => {
									e.stopPropagation();
									handleRestoreAll();
								}}
								title={t("sprites.restore")}
							>
								<Undo2 className="size-3.5" />
							</Button>
						</div>
						{excludedOpen && (
							<div className="pb-1">
								{excludes.map((id) => {
									const thumb = excludedThumbMap.get(id);
									return (
										<ContextMenu key={id}>
											<ContextMenuTrigger asChild>
												<div
													className="flex items-center gap-1.5 px-3 py-0.5 group hover:bg-accent/50"
													style={{ paddingLeft: 8 + iSize + 6 }}
													onClick={(e) => e.stopPropagation()}
												>
													{thumb && (
														<div
															className="shrink-0 rounded-sm overflow-hidden border border-white/5 opacity-40"
															style={thumb.style}
														/>
													)}
													<span className="text-xs text-muted-foreground/40 truncate flex-1 line-through">
														{id}
													</span>
													<Button
														variant="ghost"
														size="icon-xs"
														className="opacity-0 group-hover:opacity-100 shrink-0"
														onClick={() => handleRestore(id)}
														title={t("sprites.restore")}
													>
														<Undo2 className="size-3" />
													</Button>
												</div>
											</ContextMenuTrigger>
											<ContextMenuContent className="w-44">
												<ContextMenuItem onSelect={() => handleRestore(id)}>
													{t("sprites.restore")}
												</ContextMenuItem>
											</ContextMenuContent>
										</ContextMenu>
									);
								})}
							</div>
						)}
					</div>
				)}
			</div>
			<div className="flex items-center gap-2 h-6 px-2.5 border-t border-border shrink-0">
				<span className="text-xs text-muted-foreground/50 shrink-0 tabular-nums w-6 text-right">
					{thumbSize}
				</span>
				<Tooltip>
					<TooltipTrigger asChild>
						<Slider
							min={12}
							max={56}
							step={2}
							value={[thumbSize]}
							onValueChange={([v]) => setThumbSize(v)}
							className="w-full"
						/>
					</TooltipTrigger>
					<TooltipContent side="top">
						{t("sprites.thumbnailSize")}
					</TooltipContent>
				</Tooltip>
			</div>
		</div>
	);
}
