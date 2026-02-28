import { cn } from "@/lib/utils";

/** Props for {@link PanelHeader}. */
interface PanelHeaderProps {
	title: string;
	children?: React.ReactNode;
	className?: string;
	onMouseDown?: React.MouseEventHandler;
}

/** Compact panel header with an uppercase title and optional trailing content. */
export default function PanelHeader({
	title,
	children,
	className,
	onMouseDown,
}: PanelHeaderProps) {
	return (
		<div
			className={cn(
				"flex items-center justify-between h-[26px] px-2.5 border-b border-border shrink-0 bg-secondary/40",
				className,
			)}
			onMouseDown={onMouseDown}
		>
			<span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide">
				{title}
			</span>
			{children}
		</div>
	);
}
