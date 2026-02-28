import { ChevronDown, ChevronRight } from "lucide-react";
import { useState } from "react";

export function Section({
	title,
	children,
	defaultOpen = true,
}: {
	title: string;
	children: React.ReactNode;
	defaultOpen?: boolean;
}) {
	const [open, setOpen] = useState(defaultOpen);
	return (
		<div className="border-b border-border last:border-0">
			<button
				className="flex w-full items-center justify-between px-2.5 py-2 text-[11px] font-semibold text-foreground/50 hover:text-foreground/80 transition-colors"
				onClick={() => setOpen(!open)}
			>
				{title}
				{open ? (
					<ChevronDown className="size-3 opacity-60" />
				) : (
					<ChevronRight className="size-3 opacity-60" />
				)}
			</button>
			{open && <div className="px-2.5 pb-2.5 space-y-1.5">{children}</div>}
		</div>
	);
}

export function Row({
	label,
	children,
}: {
	label: string;
	children: React.ReactNode;
}) {
	return (
		<div className="flex items-center gap-2 min-h-[24px]">
			<span className="text-xs text-muted-foreground w-[116px] shrink-0 leading-tight">
				{label}
			</span>
			<div className="flex flex-1 items-center gap-1 min-w-0">{children}</div>
		</div>
	);
}
