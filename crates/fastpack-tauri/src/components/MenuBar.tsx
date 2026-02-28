import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuShortcut,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useKeyboardShortcuts } from "../hooks/useKeyboardShortcuts";
import {
	newProject,
	openFileDialog,
	openProject,
	saveFileDialog,
	savePreferences,
	saveProject,
} from "../lib/commands";
import { formatKeybind } from "../lib/keybinds";
import { useStore } from "../store";

const isMac = navigator.userAgent.includes("Mac");

export default function MenuBar() {
	const { t } = useTranslation();
	const [openMenu, setOpenMenu] = useState<"file" | "view" | null>(null);
	const setProject = useStore((s) => s.setProject);
	const setProjectPath = useStore((s) => s.setProjectPath);
	const setSheets = useStore((s) => s.setSheets);
	const setSelectedFrames = useStore((s) => s.setSelectedFrames);
	const setAnchorFrame = useStore((s) => s.setAnchorFrame);
	const setAnimPreviewOpen = useStore((s) => s.setAnimPreviewOpen);
	const setDirty = useStore((s) => s.setDirty);
	const dirty = useStore((s) => s.dirty);
	const project = useStore((s) => s.project);
	const projectPath = useStore((s) => s.projectPath);
	const setPrefsOpen = useStore((s) => s.setPrefsOpen);
	const prefs = useStore((s) => s.prefs);
	const setPrefs = useStore((s) => s.setPrefs);
	const keybinds = prefs.keybinds;

	const emptySheets = {
		sheets: [],
		log: [],
		spriteCount: 0,
		aliasCount: 0,
		overflowCount: 0,
	};

	const resetSelection = () => {
		setSelectedFrames([]);
		setAnchorFrame(null);
		setAnimPreviewOpen(false);
	};

	const handleNew = async () => {
		const p = await newProject();
		setProject(p);
		setProjectPath(null);
		setSheets(emptySheets);
		resetSelection();
	};

	const handleOpen = async () => {
		const path = await openFileDialog();
		if (!path) return;
		try {
			const p = await openProject(path);
			setProject(p);
			setProjectPath(path);
			setSheets(emptySheets);
			resetSelection();
		} catch (e) {
			console.error(e);
		}
	};

	const handleSave = async () => {
		if (!project) return;
		if (projectPath) {
			await saveProject(projectPath);
			setDirty(false);
		} else {
			await handleSaveAs();
		}
	};

	const handleSaveAs = async () => {
		if (!project) return;
		const path = await saveFileDialog("project.fpsheet");
		if (!path) return;
		await saveProject(path);
		setProjectPath(path);
		setDirty(false);
	};

	const toggleTheme = () => {
		const next = { ...prefs, dark_mode: !prefs.dark_mode };
		setPrefs(next);
		savePreferences(next);
	};

	useKeyboardShortcuts(
		isMac
			? []
			: [
					{
						key: keybinds.new_project.key,
						mod: keybinds.new_project.modifier,
						shift: keybinds.new_project.shift,
						action: handleNew,
					},
					{
						key: keybinds.open_project.key,
						mod: keybinds.open_project.modifier,
						shift: keybinds.open_project.shift,
						action: handleOpen,
					},
					{
						key: keybinds.save_project_as.key,
						mod: keybinds.save_project_as.modifier,
						shift: keybinds.save_project_as.shift,
						action: handleSaveAs,
					},
					{
						key: keybinds.save_project.key,
						mod: keybinds.save_project.modifier,
						shift: keybinds.save_project.shift,
						action: handleSave,
					},
				],
	);

	if (isMac) {
		return (
			<div className="flex items-center h-7 bg-card border-b border-border px-2 gap-1 shrink-0">
				<span className="text-xs font-semibold text-foreground px-1.5">
					FastPack
				</span>
				<span className="text-[11px] text-muted-foreground truncate max-w-40">
					{projectPath ? projectPath.split(/[\\/]/).pop() : "Untitled"}
				</span>
				{dirty && <span className="text-yellow-500 text-[10px]">*</span>}
			</div>
		);
	}

	return (
		<div className="flex items-center h-7 bg-card border-b border-border px-2 gap-1 shrink-0">
			<span className="text-xs font-semibold text-foreground px-1.5">
				FastPack
			</span>
			<span className="text-[11px] text-muted-foreground truncate max-w-40">
				{projectPath ? projectPath.split(/[\\/]/).pop() : "Untitled"}
			</span>
			{dirty && <span className="text-yellow-500 text-[10px]">*</span>}
			<div className="flex items-center ml-2 gap-0.5">
				<DropdownMenu
					open={openMenu === "file"}
					onOpenChange={(o) => setOpenMenu(o ? "file" : null)}
				>
					<DropdownMenuTrigger asChild>
						<Button
							variant="ghost"
							size="xs"
							className="h-6 px-2 text-xs"
							onMouseEnter={() => {
								if (openMenu !== null) setOpenMenu("file");
							}}
						>
							{t("menu.file")}
						</Button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="start" className="w-52">
						<DropdownMenuItem onSelect={handleNew}>
							{t("menu.newProject")}
							<DropdownMenuShortcut>
								{formatKeybind(keybinds.new_project)}
							</DropdownMenuShortcut>
						</DropdownMenuItem>
						<DropdownMenuItem onSelect={handleOpen}>
							{t("menu.openProject")}
							<DropdownMenuShortcut>
								{formatKeybind(keybinds.open_project)}
							</DropdownMenuShortcut>
						</DropdownMenuItem>
						<DropdownMenuSeparator />
						<DropdownMenuItem onSelect={handleSave} disabled={!project}>
							{t("menu.saveProject")}
							<DropdownMenuShortcut>
								{formatKeybind(keybinds.save_project)}
							</DropdownMenuShortcut>
						</DropdownMenuItem>
						<DropdownMenuItem onSelect={handleSaveAs} disabled={!project}>
							{t("menu.saveProjectAs")}
							<DropdownMenuShortcut>
								{formatKeybind(keybinds.save_project_as)}
							</DropdownMenuShortcut>
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>

				<DropdownMenu
					open={openMenu === "view"}
					onOpenChange={(o) => setOpenMenu(o ? "view" : null)}
				>
					<DropdownMenuTrigger asChild>
						<Button
							variant="ghost"
							size="xs"
							className="h-6 px-2 text-xs"
							onMouseEnter={() => {
								if (openMenu !== null) setOpenMenu("view");
							}}
						>
							{t("menu.view")}
						</Button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="start" className="w-52">
						<DropdownMenuItem onSelect={toggleTheme}>
							{prefs.dark_mode ? t("menu.lightTheme") : t("menu.darkTheme")}
						</DropdownMenuItem>
						<DropdownMenuItem onSelect={() => setPrefsOpen(true)}>
							{t("menu.preferences")}
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>
			</div>
		</div>
	);
}
