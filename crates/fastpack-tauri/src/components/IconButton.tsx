import { Button } from "@/components/ui/button";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

interface IconButtonProps {
	icon: React.ReactNode;
	tooltip: React.ReactNode;
	tooltipSide?: "top" | "bottom" | "left" | "right";
	onClick?: () => void;
	variant?: "ghost" | "outline" | "secondary";
	size?: "icon-xs" | "icon-sm";
	disabled?: boolean;
	className?: string;
}

export default function IconButton({
	icon,
	tooltip,
	tooltipSide,
	onClick,
	variant = "ghost",
	size = "icon-xs",
	disabled,
	className,
}: IconButtonProps) {
	return (
		<Tooltip>
			<TooltipTrigger asChild>
				<Button
					variant={variant}
					size={size}
					onClick={onClick}
					disabled={disabled}
					className={cn(className)}
				>
					{icon}
				</Button>
			</TooltipTrigger>
			<TooltipContent side={tooltipSide}>{tooltip}</TooltipContent>
		</Tooltip>
	);
}
