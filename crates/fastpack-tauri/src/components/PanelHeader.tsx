import { cn } from "@/lib/utils";

interface PanelHeaderProps {
	title: string;
	children?: React.ReactNode;
	className?: string;
	onMouseDown?: React.MouseEventHandler;
}

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
