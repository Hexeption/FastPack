import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { pack, updateProject } from "../lib/commands";
import { useStore } from "../store";
import type { Project } from "../types";
import PanelHeader from "./PanelHeader";
import LayoutSection from "./settings/LayoutSection";
import OutputSection from "./settings/OutputSection";
import SpriteSection from "./settings/SpriteSection";
import VariantSection from "./settings/VariantSection";

export default function SettingsPanel() {
	const { t } = useTranslation();
	const project = useStore((s) => s.project);
	const setProject = useStore((s) => s.setProject);
	const setDirty = useStore((s) => s.setDirty);
	const packTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

	if (!project)
		return (
			<div className="flex flex-col w-full h-full border-l border-border bg-card" />
		);

	const update = (p: Project) => {
		setProject(p);
		setDirty(true);
		updateProject(p);
		if (packTimer.current) clearTimeout(packTimer.current);
		packTimer.current = setTimeout(() => {
			pack().catch(console.error);
		}, 300);
	};

	// Persist without repacking — for output settings that don't affect layout
	const save = (p: Project) => {
		setProject(p);
		setDirty(true);
		updateProject(p);
	};

	return (
		<div className="flex flex-col w-full h-full border-l border-border bg-card overflow-hidden">
			<PanelHeader title={t("settings.title")} />
			<div className="flex-1 overflow-y-auto">
				<OutputSection project={project} update={update} save={save} />
				<LayoutSection project={project} update={update} save={save} />
				<SpriteSection project={project} update={update} save={save} />
				<VariantSection project={project} update={update} save={save} />
			</div>
		</div>
	);
}
