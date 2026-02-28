import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { SHORTCUT_ORDER, useKeybindCapture } from "../hooks/useKeybindCapture";
import {
	checkCliInstalled,
	installCli,
	openConfigFolder,
	savePreferences,
} from "../lib/commands";
import { formatKeybind } from "../lib/keybinds";
import { useStore } from "../store";
import type { AnimBg, KeybindsConfig, Preferences } from "../types";
import DefaultsTab from "./DefaultsTab";
import UpdatesTab from "./UpdatesTab";

/** Modal preferences dialog with tabs for general settings, keybinds, project defaults, and updates. */
export default function PreferencesDialog() {
	const { t } = useTranslation();
	const prefs = useStore((s) => s.prefs);
	const setPrefs = useStore((s) => s.setPrefs);
	const prefsOpen = useStore((s) => s.prefsOpen);
	const setPrefsOpen = useStore((s) => s.setPrefsOpen);

	const { capturing, setCapturing, hasDuplicate } = useKeybindCapture();

	const [cliInstalled, setCliInstalled] = useState(false);

	useEffect(() => {
		if (prefsOpen) {
			checkCliInstalled()
				.then(setCliInstalled)
				.catch(() => setCliInstalled(false));
		}
	}, [prefsOpen]);

	const handleInstallCli = useCallback(() => {
		installCli()
			.then(() => setCliInstalled(true))
			.catch(console.error);
	}, []);

	const SHORTCUT_LABELS: Record<keyof KeybindsConfig, string> = {
		new_project: t("prefs.keybindNewProject"),
		open_project: t("prefs.keybindOpenProject"),
		save_project: t("prefs.keybindSaveProject"),
		save_project_as: t("prefs.keybindSaveProjectAs"),
		anim_preview: t("prefs.keybindAnimPreview"),
	};

	const onOpenChange = (o: boolean) => {
		if (!o) {
			setPrefsOpen(false);
			setCapturing(null);
		}
	};

	const update = (patch: Partial<Preferences>) => {
		const next = { ...prefs, ...patch };
		setPrefs(next);
		savePreferences(next).catch(console.error);
	};

	return (
		<Dialog open={prefsOpen} onOpenChange={onOpenChange}>
			<DialogContent className="w-auto sm:min-w-[420px] sm:max-w-[560px]">
				<DialogHeader>
					<DialogTitle>{t("prefs.title")}</DialogTitle>
				</DialogHeader>

				<Tabs defaultValue="general">
					<TabsList className="w-full">
						<TabsTrigger value="general" className="flex-1">
							{t("prefs.general")}
						</TabsTrigger>
						<TabsTrigger value="keybinds" className="flex-1">
							{t("prefs.keybinds")}
						</TabsTrigger>
						<TabsTrigger value="defaults" className="flex-1">
							{t("prefs.defaults")}
						</TabsTrigger>
						<TabsTrigger value="updates" className="flex-1">
							{t("prefs.updates")}
						</TabsTrigger>
					</TabsList>

					<TabsContent value="general" className="space-y-4 py-1 mt-3">
						<div className="flex items-center justify-between">
							<Label htmlFor="pref-dark" className="text-sm">
								{t("prefs.darkMode")}
							</Label>
							<Switch
								id="pref-dark"
								checked={prefs.dark_mode}
								onCheckedChange={(c) => update({ dark_mode: c })}
							/>
						</div>
						<div className="flex items-center justify-between">
							<Label htmlFor="pref-updates" className="text-sm">
								{t("prefs.checkUpdatesOnStartup")}
							</Label>
							<Switch
								id="pref-updates"
								checked={prefs.auto_check_updates}
								onCheckedChange={(c) => update({ auto_check_updates: c })}
							/>
						</div>
						<div className="flex items-center justify-between gap-4">
							<Label className="text-sm shrink-0">{t("prefs.language")}</Label>
							<Select
								value={prefs.language}
								onValueChange={(v) =>
									update({ language: v as Preferences["language"] })
								}
							>
								<SelectTrigger className="h-8 text-sm w-[140px]">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="En">English</SelectItem>
									<SelectItem value="Fr">Français</SelectItem>
									<SelectItem value="Es">Español</SelectItem>
									<SelectItem value="De">Deutsch</SelectItem>
									<SelectItem value="It">Italiano</SelectItem>
									<SelectItem value="Pt">Português</SelectItem>
									<SelectItem value="Ja">日本語</SelectItem>
									<SelectItem value="Zh">中文（简体）</SelectItem>
									<SelectItem value="Ko">한국어</SelectItem>
								</SelectContent>
							</Select>
						</div>
						<div className="flex items-center justify-between gap-4">
							<Label className="text-sm shrink-0">{t("prefs.uiScale")}</Label>
							<Select
								value={String(prefs.ui_scale)}
								onValueChange={(v) => update({ ui_scale: Number(v) })}
							>
								<SelectTrigger className="h-8 text-sm w-[140px]">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="0.75">75%</SelectItem>
									<SelectItem value="1">100%</SelectItem>
									<SelectItem value="1.25">125%</SelectItem>
									<SelectItem value="1.5">150%</SelectItem>
									<SelectItem value="1.75">175%</SelectItem>
									<SelectItem value="2">200%</SelectItem>
								</SelectContent>
							</Select>
						</div>
						<div className="flex items-center justify-between gap-4">
							<Label className="text-sm shrink-0">
								{t("prefs.atlasZoomSpeed")}
							</Label>
							<Select
								value={String(prefs.atlas_zoom_speed ?? 1)}
								onValueChange={(v) => update({ atlas_zoom_speed: Number(v) })}
							>
								<SelectTrigger className="h-8 text-sm w-[140px]">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="0.5">{t("prefs.zoomSlow")}</SelectItem>
									<SelectItem value="1">{t("prefs.zoomNormal")}</SelectItem>
									<SelectItem value="1.5">{t("prefs.zoomFast")}</SelectItem>
									<SelectItem value="2">{t("prefs.zoomVeryFast")}</SelectItem>
								</SelectContent>
							</Select>
						</div>
						<div className="flex items-center justify-between">
							<Label htmlFor="pref-invert-scroll" className="text-sm">
								{t("prefs.atlasInvertScroll")}
							</Label>
							<Switch
								id="pref-invert-scroll"
								checked={prefs.atlas_invert_scroll ?? false}
								onCheckedChange={(c) => update({ atlas_invert_scroll: c })}
							/>
						</div>
						<div className="flex items-center justify-between gap-4">
							<Label className="text-sm shrink-0">
								{t("prefs.animPreviewFps")}
							</Label>
							<Input
								type="number"
								min={1}
								max={120}
								className="h-8 text-sm w-[140px]"
								value={prefs.anim_preview_fps ?? 24}
								onChange={(e) => {
									const v = Number(e.target.value);
									if (v >= 1 && v <= 120) update({ anim_preview_fps: v });
								}}
							/>
						</div>
						<div className="flex items-center justify-between gap-4">
							<Label className="text-sm shrink-0">
								{t("prefs.animPreviewBg")}
							</Label>
							<Select
								value={prefs.anim_preview_bg ?? "checker"}
								onValueChange={(v) => update({ anim_preview_bg: v as AnimBg })}
							>
								<SelectTrigger className="h-8 text-sm w-[140px]">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="checker">
										{t("prefs.animBgChecker")}
									</SelectItem>
									<SelectItem value="black">
										{t("prefs.animBgBlack")}
									</SelectItem>
									<SelectItem value="white">
										{t("prefs.animBgWhite")}
									</SelectItem>
								</SelectContent>
							</Select>
						</div>
						<div className="pt-1 border-t border-border">
							<Button
								variant="ghost"
								size="sm"
								className="h-7 px-2 text-xs text-muted-foreground w-full justify-start"
								onClick={() => openConfigFolder().catch(console.error)}
							>
								{t("prefs.openConfigFolder")}
							</Button>
							<Button
								variant="ghost"
								size="sm"
								className="h-7 px-2 text-xs text-muted-foreground w-full justify-start"
								disabled={cliInstalled}
								onClick={handleInstallCli}
							>
								{cliInstalled ? t("prefs.cliInstalled") : t("prefs.installCli")}
							</Button>
						</div>
					</TabsContent>

					<TabsContent value="keybinds" className="py-1 space-y-0.5 mt-3">
						{SHORTCUT_ORDER.map((id) => {
							const isCapturing = capturing === id;
							const isDuplicate = !isCapturing && hasDuplicate(id);
							return (
								<div
									key={id}
									className="flex items-center justify-between py-1 px-1 rounded"
								>
									<span className="text-sm text-foreground flex items-center gap-1.5">
										{SHORTCUT_LABELS[id]}
										{isDuplicate && (
											<span
												className="text-xs text-destructive"
												title={t("prefs.duplicateShortcut")}
											>
												!
											</span>
										)}
									</span>
									<button
										onClick={() => setCapturing(isCapturing ? null : id)}
										className={
											"px-2 py-0.5 text-xs font-mono rounded border text-left min-w-[72px] transition-colors " +
											(isCapturing
												? "bg-primary/10 border-primary text-primary animate-pulse"
												: "bg-muted border-border text-muted-foreground hover:border-foreground/40 cursor-pointer")
										}
									>
										{isCapturing
											? t("prefs.pressAKey")
											: formatKeybind(prefs.keybinds[id])}
									</button>
								</div>
							);
						})}
						<p className="text-xs text-muted-foreground pt-2 px-1">
							{t("prefs.keybindHint")}
						</p>
					</TabsContent>

					<TabsContent value="defaults" className="mt-3">
						<DefaultsTab
							config={prefs.default_config}
							onChange={(c) => update({ default_config: c })}
						/>
					</TabsContent>

					<TabsContent value="updates" className="mt-3">
						<UpdatesTab />
					</TabsContent>
				</Tabs>
			</DialogContent>
		</Dialog>
	);
}
