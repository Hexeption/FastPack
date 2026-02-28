import {
	ResizableHandle,
	ResizablePanel,
	ResizablePanelGroup,
} from "@/components/ui/resizable";
import AnimPreview from "./components/AnimPreview";
import AtlasPreview from "./components/AtlasPreview";
import MenuBar from "./components/MenuBar";
import OutputLog from "./components/OutputLog";
import PreferencesDialog from "./components/PreferencesDialog";
import SettingsPanel from "./components/SettingsPanel";
import StatusBar from "./components/StatusBar";
import SpriteList from "./components/sprite-list";
import Toolbar from "./components/Toolbar";
import { useAppInit } from "./hooks/useAppInit";
import { useDrop } from "./hooks/useDrop";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useLayoutPersistence } from "./hooks/useLayoutPersistence";
import { useMenuEvents } from "./hooks/useMenuEvents";
import { usePack } from "./hooks/usePack";
import { pack, updateProject } from "./lib/commands";
import { useStore } from "./store";

function excludeSelected() {
	const state = useStore.getState();
	const { selectedFrames, project } = state;
	if (selectedFrames.length === 0 || !project) return;
	const existing = new Set(project.excludes ?? []);
	for (const id of selectedFrames) existing.add(id);
	const updated = { ...project, excludes: [...existing] };
	state.setProject(updated);
	state.setSelectedFrames([]);
	state.setAnchorFrame(null);
	state.setDirty(true);
	updateProject(updated)
		.then(() => pack())
		.catch(console.error);
}

export default function App() {
	const prefs = useStore((s) => s.prefs);
	const setAnimPreviewOpen = useStore((s) => s.setAnimPreviewOpen);

	const {
		spritesRef,
		settingsRef,
		outputRef,
		groupVRef,
		groupHRef,
		onLayoutChangedV,
		onLayoutChangedH,
	} = useLayoutPersistence();

	useAppInit();
	usePack();
	useDrop();
	useMenuEvents();

	useKeyboardShortcuts([
		{
			key: prefs.keybinds.anim_preview.key,
			mod: prefs.keybinds.anim_preview.modifier,
			shift: prefs.keybinds.anim_preview.shift,
			action: () => {
				const { selectedFrames } = useStore.getState();
				if (selectedFrames.length >= 2) setAnimPreviewOpen(true);
			},
		},
		{ key: "Delete", action: excludeSelected },
		{ key: "Backspace", action: excludeSelected },
	]);

	return (
		<div className="flex flex-col h-screen overflow-hidden bg-background text-foreground">
			<MenuBar />
			<Toolbar />
			<ResizablePanelGroup
				groupRef={groupVRef}
				onLayoutChanged={onLayoutChangedV}
				orientation="vertical"
				className="flex-1 min-h-0"
			>
				<ResizablePanel id="main" defaultSize={78} minSize={40}>
					<ResizablePanelGroup
						groupRef={groupHRef}
						onLayoutChanged={onLayoutChangedH}
						orientation="horizontal"
						className="h-full"
					>
						<ResizablePanel
							id="sprites"
							panelRef={spritesRef}
							defaultSize={300}
							minSize={150}
							collapsible
							collapsedSize={0}
						>
							<SpriteList />
						</ResizablePanel>
						<ResizableHandle />
						<ResizablePanel id="atlas">
							<AtlasPreview />
						</ResizablePanel>
						<ResizableHandle />
						<ResizablePanel
							id="settings"
							panelRef={settingsRef}
							defaultSize={300}
							minSize={300}
							collapsible
							collapsedSize={0}
						>
							<SettingsPanel />
						</ResizablePanel>
					</ResizablePanelGroup>
				</ResizablePanel>
				<ResizableHandle />
				<ResizablePanel
					id="output"
					panelRef={outputRef}
					defaultSize={22}
					minSize={100}
					collapsible
					collapsedSize={0}
				>
					<OutputLog />
				</ResizablePanel>
			</ResizablePanelGroup>
			<PreferencesDialog />
			<AnimPreview />
			<StatusBar />
		</div>
	);
}
