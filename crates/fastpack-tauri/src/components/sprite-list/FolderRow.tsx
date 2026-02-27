import { ChevronDown, ChevronRight, Folder, FolderOpen } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
	ContextMenu,
	ContextMenuContent,
	ContextMenuSub,
	ContextMenuSubContent,
	ContextMenuSubTrigger,
	ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { FOLDER_SWATCHES } from "../../lib/folder-colors";
import type { FolderNode } from "../../lib/tree";
import FileRow, { iconSize, type ThumbInfo } from "./FileRow";

export default function FolderRow({
	node,
	indent,
	isOpen,
	onToggle,
	openFolders,
	onToggleFolder,
	selectedFrames,
	onSelect,
	onZoom,
	onExclude,
	folderColors,
	onSetFolderColor,
	sourceColor,
	thumbMap,
	thumbSize,
}: {
	node: FolderNode;
	indent: number;
	isOpen: boolean;
	onToggle: () => void;
	openFolders: Set<string>;
	onToggleFolder: (path: string) => void;
	selectedFrames: string[];
	onSelect: (id: string, ctrlKey: boolean, shiftKey: boolean) => void;
	onZoom: (id: string) => void;
	onExclude?: (id: string) => void;
	folderColors?: Record<string, string>;
	onSetFolderColor?: (key: string, color: string | null) => void;
	sourceColor: string;
	thumbMap: Map<string, ThumbInfo>;
	thumbSize: number;
}) {
	const { t } = useTranslation();
	const ico = iconSize(thumbSize);
	const color = folderColors?.[node.path];

	const navColor = color ?? sourceColor;

	const row = (
		<button
			className="flex items-center gap-1 w-full py-px pr-2.5 hover:bg-accent/50 text-left"
			style={{ paddingLeft: `${indent + 4}px` }}
			data-nav-id={`__folder__:${node.path}`}
			data-nav-color={navColor}
			onClick={(e) => {
				e.stopPropagation();
				onToggle();
			}}
		>
			{isOpen ? (
				<ChevronDown size={ico} className="shrink-0 text-muted-foreground/50" />
			) : (
				<ChevronRight
					size={ico}
					className="shrink-0 text-muted-foreground/50"
				/>
			)}
			{isOpen ? (
				<FolderOpen
					size={ico}
					className="shrink-0"
					style={{ color: color ?? "var(--muted-foreground)" }}
				/>
			) : (
				<Folder
					size={ico}
					className="shrink-0"
					style={{ color: color ?? "var(--muted-foreground)" }}
				/>
			)}
			<span className="text-xs text-foreground/80 font-medium truncate">
				{node.name}
			</span>
		</button>
	);

	return (
		<>
			{onSetFolderColor ? (
				<ContextMenu>
					<ContextMenuTrigger asChild>{row}</ContextMenuTrigger>
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
											onClick={() => onSetFolderColor(node.path, value)}
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
								{color !== undefined && (
									<>
										<div className="-mx-2 my-2 h-px bg-border" />
										<button
											className="w-full text-left text-xs text-muted-foreground/60 px-0.5 hover:text-foreground transition-colors"
											onClick={() => onSetFolderColor(node.path, null)}
										>
											{t("sprites.resetColor")}
										</button>
									</>
								)}
							</ContextMenuSubContent>
						</ContextMenuSub>
					</ContextMenuContent>
				</ContextMenu>
			) : (
				row
			)}
			{isOpen &&
				node.children.map((child) =>
					child.kind === "folder" ? (
						<FolderRow
							key={child.path}
							node={child}
							indent={indent + 14}
							isOpen={openFolders.has(child.path)}
							onToggle={() => onToggleFolder(child.path)}
							openFolders={openFolders}
							onToggleFolder={onToggleFolder}
							selectedFrames={selectedFrames}
							onSelect={onSelect}
							onZoom={onZoom}
							onExclude={onExclude}
							folderColors={folderColors}
							onSetFolderColor={onSetFolderColor}
							sourceColor={sourceColor}
							thumbMap={thumbMap}
							thumbSize={thumbSize}
						/>
					) : (
						<FileRow
							key={child.frame.id}
							node={child}
							indent={indent + 14}
							isSelected={selectedFrames.includes(child.frame.id)}
							onSelect={onSelect}
							onZoom={onZoom}
							onExclude={onExclude}
							thumb={thumbMap.get(child.frame.id)}
						/>
					),
				)}
		</>
	);
}
