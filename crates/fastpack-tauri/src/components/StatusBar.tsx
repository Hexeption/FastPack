import { useTranslation } from "react-i18next";
import { useStore } from "../store";

export default function StatusBar() {
	const { t } = useTranslation();
	const sheets = useStore((s) => s.sheets);
	const spriteCount = useStore((s) => s.spriteCount);
	const aliasCount = useStore((s) => s.aliasCount);
	const overflowCount = useStore((s) => s.overflowCount);
	const selectedFrames = useStore((s) => s.selectedFrames);
	const isPacking = useStore((s) => s.isPacking);
	const isWatching = useStore((s) => s.isWatching);

	const dims =
		sheets.length > 0
			? sheets.map((s) => `${s.width}\u00d7${s.height}`).join(", ")
			: null;

	return (
		<div className="flex items-center h-[22px] bg-card border-t border-border px-2.5 gap-3 shrink-0 text-[10px] text-muted-foreground tabular-nums select-none">
			{dims && <span>{dims}</span>}
			{spriteCount > 0 && (
				<span>
					{spriteCount} {t("status.sprites")}
				</span>
			)}
			{aliasCount > 0 && (
				<span>
					{aliasCount} {t("status.aliases")}
				</span>
			)}
			{overflowCount > 0 && (
				<span className="text-destructive">
					{overflowCount} {t("status.overflow")}
				</span>
			)}
			{selectedFrames.length > 0 && (
				<span>
					{selectedFrames.length} {t("status.selected")}
				</span>
			)}
			<div className="ml-auto flex items-center gap-2">
				{isPacking && (
					<span className="text-primary">{t("status.packing")}</span>
				)}
				{isWatching && (
					<span className="flex items-center gap-1">
						<span className="size-1.5 rounded-full bg-green-500 animate-pulse" />
						{t("status.watching")}
					</span>
				)}
			</div>
		</div>
	);
}
