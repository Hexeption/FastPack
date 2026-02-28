import { ImageIcon, Link2 } from "lucide-react";
import { type CSSProperties, memo } from "react";
import { useTranslation } from "react-i18next";
import {
	ContextMenu,
	ContextMenuContent,
	ContextMenuItem,
	ContextMenuSeparator,
	ContextMenuTrigger,
} from "@/components/ui/context-menu";
import type { FileNode } from "../../lib/tree";

/** Default thumbnail pixel size. */
export const THUMB_DEFAULT = 40;

/** Thumbnail CSS for a single frame entry. */
export type ThumbInfo = { style: CSSProperties };

/** Scales icon size proportionally to the thumbnail slider value. */
export function iconSize(thumbSize: number): number {
	return Math.max(10, Math.round(thumbSize * 0.4));
}

/** Inline sprite thumbnail with checkerboard background. */
function FrameThumb({ thumb }: { thumb: ThumbInfo }) {
	return (
		<div
			className="shrink-0 rounded-sm overflow-hidden border border-white/5"
			style={thumb.style}
		/>
	);
}

/** Single sprite file row. Shows thumbnail, name, alias badge, dimensions, and a context menu for zoom/exclude. */
const FileRow = memo(function FileRow({
	node,
	indent,
	isSelected,
	onSelect,
	onZoom,
	onExclude,
	thumb,
}: {
	node: FileNode;
	indent: number;
	isSelected: boolean;
	onSelect: (id: string, ctrlKey: boolean, shiftKey: boolean) => void;
	onZoom: (id: string) => void;
	onExclude?: (id: string) => void;
	thumb: ThumbInfo | undefined;
}) {
	const { t } = useTranslation();
	const id = node.frame.id;

	const row = (
		<div
			data-frame-id={id}
			data-nav-id={id}
			className={`flex items-center justify-between py-0.5 pr-2.5 hover:bg-accent/50 group cursor-pointer ${isSelected ? "bg-primary/15 border-l-2 border-primary" : ""}`}
			style={{ paddingLeft: `${indent + 8}px` }}
			title={id}
			onClick={(e) => {
				e.stopPropagation();
				onSelect(id, e.ctrlKey || e.metaKey, e.shiftKey);
			}}
			onDoubleClick={(e) => {
				e.stopPropagation();
				onZoom(id);
			}}
		>
			<span className="flex items-center gap-1.5 min-w-0">
				{thumb ? (
					<FrameThumb thumb={thumb} />
				) : (
					<ImageIcon className="size-3 shrink-0 text-muted-foreground/50" />
				)}
				<span
					className={`text-xs truncate ${isSelected ? "text-foreground font-medium" : "text-foreground"}`}
				>
					{node.name}
				</span>
				{node.frame.alias_of !== null && (
					<span
						title={`Alias of: ${node.frame.alias_of}`}
						className="shrink-0 flex items-center"
					>
						<Link2 className="size-2.5" />
					</span>
				)}
			</span>
			<span className="text-xs text-muted-foreground/60 shrink-0 ml-1 tabular-nums">
				{node.frame.w}&times;{node.frame.h}
			</span>
		</div>
	);

	if (!onExclude) return row;

	return (
		<ContextMenu>
			<ContextMenuTrigger asChild>{row}</ContextMenuTrigger>
			<ContextMenuContent className="w-44">
				<ContextMenuItem onSelect={() => onZoom(id)}>
					{t("sprites.zoomToFrame")}
				</ContextMenuItem>
				<ContextMenuSeparator />
				<ContextMenuItem
					onSelect={() => onExclude(id)}
					className="text-destructive focus:text-destructive"
				>
					{t("sprites.exclude")}
				</ContextMenuItem>
			</ContextMenuContent>
		</ContextMenu>
	);
});

export default FileRow;
