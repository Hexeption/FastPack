import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { applyUpdate, checkForUpdate, downloadUpdate } from "../lib/commands";
import type { ReleaseInfo } from "../types";

export default function UpdatesTab() {
	const { t } = useTranslation();
	const [status, setStatus] = useState<
		"idle" | "checking" | "up-to-date" | "available" | "error"
	>("idle");
	const [info, setInfo] = useState<ReleaseInfo | null>(null);
	const [downloading, setDownloading] = useState(false);

	const check = async () => {
		setStatus("checking");
		try {
			const result = await checkForUpdate();
			if (result) {
				setInfo(result);
				setStatus("available");
			} else {
				setStatus("up-to-date");
			}
		} catch {
			setStatus("error");
		}
	};

	const download = async () => {
		if (!info) return;
		setDownloading(true);
		try {
			await downloadUpdate(info.asset_url);
			await applyUpdate();
		} catch (e) {
			console.error(e);
		} finally {
			setDownloading(false);
		}
	};

	return (
		<div className="py-1 space-y-3">
			<div className="text-xs text-muted-foreground">
				{t("updates.currentVersion")}{" "}
				<span className="font-mono text-foreground">0.26.0</span>
			</div>
			<div className="flex items-center gap-2">
				<Button
					variant="outline"
					size="sm"
					onClick={check}
					disabled={status === "checking"}
				>
					{status === "checking"
						? t("updates.checking")
						: t("updates.checkForUpdates")}
				</Button>
				{status === "up-to-date" && (
					<span className="text-xs text-muted-foreground">
						{t("updates.upToDate")}
					</span>
				)}
				{status === "error" && (
					<span className="text-xs text-destructive">
						{t("updates.checkFailed")}
					</span>
				)}
			</div>
			{status === "available" && info && (
				<div className="rounded-md border border-border p-3 space-y-2">
					<div className="text-sm font-medium">
						{t("updates.versionAvailable", { version: info.version })}
					</div>
					{info.notes && (
						<p className="text-xs text-muted-foreground leading-relaxed">
							{info.notes}
						</p>
					)}
					<Button size="sm" onClick={download} disabled={downloading}>
						{downloading
							? t("updates.downloading")
							: t("updates.downloadAndInstall")}
					</Button>
				</div>
			)}
		</div>
	);
}
