import { ChevronDown, ChevronRight, Folder, FolderOpen, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
	ContextMenu,
	ContextMenuContent,
	ContextMenuItem,
	ContextMenuSeparator,
	ContextMenuSub,
	ContextMenuSubContent,
	ContextMenuSubTrigger,
	ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { FOLDER_SWATCHES, SOURCE_DEFAULT_COLOR } from "../../lib/folder-colors";
import type { TreeNode } from "../../lib/tree";
import FileRow, { iconSize, type ThumbInfo } from "./FileRow";
import FolderRow from "./FolderRow";

interface SourceRowProps {
	srcName: string;
	srcPath: string;
	srcFrameCount: number;
	tree: TreeNode[];
	isOpen: boolean;
	onToggle: () => void;
	navKey: string;
	openFolders: Set<string>;
	onToggleFolder: (path: string) => void;
	onRemove: () => void;
	onExclude?: (id: string) => void;
	folderColors: Record<string, string>;
	onSetFolderColor: (key: string, color: string | null) => void;
	selectedFrames: string[];
	onSelect: (id: string, ctrlKey: boolean, shiftKey: boolean) => void;
	onZoom: (id: string) => void;
	thumbMap: Map<string, ThumbInfo>;
	thumbSize: number;
	allFramesEmpty: boolean;
}

export default function SourceRow({
	srcName,
	srcPath,
	srcFrameCount,
	tree,
	isOpen,
	onToggle,
	navKey,
	openFolders,
	onToggleFolder,
	onRemove,
	onExclude,
	folderColors,
	onSetFolderColor,
	selectedFrames,
	onSelect,
	onZoom,
	thumbMap,
	thumbSize,
	allFramesEmpty,
}: SourceRowProps) {
	const { t } = useTranslation();
	const iSize = iconSize(thumbSize);
	const color = folderColors[srcPath] ?? SOURCE_DEFAULT_COLOR;

	const header = (
		<div
			data-nav-id={navKey}
			data-nav-color={color}
			className="flex items-center justify-between px-2.5 py-1 hover:bg-accent/80 group select-none border-b border-border/50 bg-secondary/50 border-l-2"
			style={{ borderLeftColor: color }}
		>
			<button
				className="flex items-center gap-1.5 min-w-0 flex-1 text-left"
				onClick={(e) => {
					e.stopPropagation();
					onToggle();
				}}
			>
				{isOpen ? (
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
				{isOpen ? (
					<FolderOpen size={iSize} className="shrink-0" style={{ color }} />
				) : (
					<Folder size={iSize} className="shrink-0" style={{ color }} />
				)}
				<span className="text-xs font-medium text-foreground truncate">
					{srcName}
				</span>
				{srcFrameCount > 0 && (
					<span className="text-xs text-muted-foreground/60 shrink-0">
						({srcFrameCount})
					</span>
				)}
			</button>
			<Button
				variant="ghost"
				size="icon-xs"
				className="opacity-0 group-hover:opacity-100 shrink-0"
				onClick={(e) => {
					e.stopPropagation();
					onRemove();
				}}
				title={t("sprites.removeSource")}
			>
				<X className="size-3.5" />
			</Button>
		</div>
	);

	return (
		<div>
			<ContextMenu>
				<ContextMenuTrigger asChild>{header}</ContextMenuTrigger>
				<ContextMenuContent className="w-44">
					<ContextMenuSub>
						<ContextMenuSubTrigger>
							{t("sprites.setColor")}
						</ContextMenuSubTrigger>
						<ContextMenuSubContent className="p-2 w-auto">
							<div className="grid grid-cols-5 gap-1.5">
								{FOLDER_SWATCHES.map(({ value, label }) => (
									<button
										key={value}
										title={label}
										onClick={() => onSetFolderColor(srcPath, value)}
										className="size-5 rounded-full transition-transform hover:scale-110 focus:outline-none"
										style={{
											backgroundColor: value,
											boxShadow:
												color === value
													? `0 0 0 2px white, 0 0 0 3.5px ${value}`
													: "inset 0 0 0 1px rgba(0,0,0,0.2)",
										}}
									/>
								))}
							</div>
							{folderColors[srcPath] !== undefined && (
								<>
									<div className="-mx-2 my-2 h-px bg-border" />
									<button
										className="w-full text-left text-xs text-muted-foreground/60 px-0.5 hover:text-foreground transition-colors"
										onClick={() => onSetFolderColor(srcPath, null)}
									>
										{t("sprites.resetColor")}
									</button>
								</>
							)}
						</ContextMenuSubContent>
					</ContextMenuSub>
					<ContextMenuSeparator />
					<ContextMenuItem
						onSelect={onRemove}
						className="text-destructive focus:text-destructive"
					>
						{t("sprites.removeSource")}
					</ContextMenuItem>
				</ContextMenuContent>
			</ContextMenu>
			{isOpen && (
				<div className="pb-1">
					{allFramesEmpty ? (
						<span className="block px-4 py-1 text-xs text-muted-foreground italic">
							{t("sprites.packToSee")}
						</span>
					) : tree.length === 0 ? (
						<span className="block px-4 py-1 text-xs text-muted-foreground italic">
							{t("sprites.noSpritesFound")}
						</span>
					) : (
						tree.map((node) =>
							node.kind === "folder" ? (
								<FolderRow
									key={node.path}
									node={node}
									indent={8}
									isOpen={openFolders.has(node.path)}
									onToggle={() => onToggleFolder(node.path)}
									openFolders={openFolders}
									onToggleFolder={onToggleFolder}
									selectedFrames={selectedFrames}
									onSelect={onSelect}
									onZoom={onZoom}
									onExclude={onExclude}
									folderColors={folderColors}
									onSetFolderColor={onSetFolderColor}
									sourceColor={color}
									thumbMap={thumbMap}
									thumbSize={thumbSize}
								/>
							) : (
								<FileRow
									key={node.frame.id}
									node={node}
									indent={8}
									isSelected={selectedFrames.includes(node.frame.id)}
									onSelect={onSelect}
									onZoom={onZoom}
									onExclude={onExclude}
									thumb={thumbMap.get(node.frame.id)}
								/>
							),
						)
					)}
				</div>
			)}
		</div>
	);
}
