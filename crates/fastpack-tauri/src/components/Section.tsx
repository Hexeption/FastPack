import { ChevronDown, ChevronRight } from "lucide-react";
import { useState } from "react";
import { Label } from "@/components/ui/label";

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
				className="flex w-full items-center justify-between px-2.5 py-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wide hover:text-foreground transition-colors"
				onClick={() => setOpen(!open)}
			>
				{title}
				{open ? (
					<ChevronDown className="size-3.5" />
				) : (
					<ChevronRight className="size-3.5" />
				)}
			</button>
			{open && <div className="px-2.5 pb-1.5 space-y-1">{children}</div>}
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
		<div className="flex items-center justify-between gap-1.5 min-h-[22px]">
			<Label className="text-xs text-muted-foreground shrink-0">{label}</Label>
			<div className="flex items-center gap-1">{children}</div>
		</div>
	);
}
