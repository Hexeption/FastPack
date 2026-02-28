import { AlertTriangle, Info, Trash2, XCircle } from "lucide-react";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { useStore } from "../store";
import PanelHeader from "./PanelHeader";

/** Scrollable log panel that auto-scrolls to new entries when pinned to the bottom. */
export default function OutputLog() {
	const { t } = useTranslation();
	const log = useStore((s) => s.log);
	const setLog = useStore((s) => s.setLog);
	const scrollRef = useRef<HTMLDivElement>(null);
	const pinnedRef = useRef(true);

	const handleScroll = () => {
		const el = scrollRef.current;
		if (!el) return;
		pinnedRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 32;
	};

	// biome-ignore lint/correctness/useExhaustiveDependencies: log triggers scroll on new entries
	useEffect(() => {
		if (!pinnedRef.current) return;
		const el = scrollRef.current;
		if (el) el.scrollTop = el.scrollHeight;
	}, [log]);

	return (
		<div className="flex flex-col h-full bg-card border-t border-border">
			<PanelHeader title={t("outputLog.title")} className="h-7">
				<Button
					variant="ghost"
					size="icon-xs"
					onClick={() => setLog([])}
					title={t("outputLog.clearLog")}
				>
					<Trash2 className="size-3" />
				</Button>
			</PanelHeader>
			<div
				ref={scrollRef}
				onScroll={handleScroll}
				className="flex-1 overflow-y-auto px-2 py-1 min-h-0"
			>
				{log.length === 0 ? (
					<p className="text-xs text-muted-foreground/40 italic px-1 py-0.5">
						{t("outputLog.noOutput")}
					</p>
				) : (
					log.map((entry, i) => {
						const isWarn = entry.level === "warn";
						const isError = entry.level === "error";
						return (
							<div
								key={i}
								className={`font-mono text-[10px] flex items-start gap-1.5 py-px px-1 ${i % 2 === 0 ? "" : "bg-muted/30"}`}
							>
								<span className="shrink-0 text-muted-foreground/50 mt-px w-14 tabular-nums">
									{entry.time}
								</span>
								<span className="shrink-0 mt-[1px]">
									{isError ? (
										<XCircle className="size-3 text-destructive" />
									) : isWarn ? (
										<AlertTriangle className="size-3 text-yellow-500" />
									) : (
										<Info className="size-3 text-emerald-500" />
									)}
								</span>
								<span
									className={`break-words min-w-0 leading-relaxed ${
										isError
											? "text-destructive"
											: isWarn
												? "text-yellow-500"
												: "text-emerald-500/80"
									}`}
								>
									{entry.message}
								</span>
							</div>
						);
					})
				)}
			</div>
		</div>
	);
}
