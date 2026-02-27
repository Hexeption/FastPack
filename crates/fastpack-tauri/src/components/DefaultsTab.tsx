import type { PackerConfig, Project } from "../types";
import LayoutSection from "./settings/LayoutSection";
import OutputSection from "./settings/OutputSection";
import SpriteSection from "./settings/SpriteSection";
import VariantSection from "./settings/VariantSection";

interface Props {
	config: PackerConfig;
	onChange: (c: PackerConfig) => void;
}

function toProject(c: PackerConfig): Project {
	return { ...c, sources: [], folder_colors: {} };
}

function fromProject(p: Project): PackerConfig {
	const { sources: _sources, folder_colors: _fc, ...config } = p;
	return config as PackerConfig;
}

export default function DefaultsTab({ config, onChange }: Props) {
	const fakeProject = toProject(config);
	const update = (p: Project) => onChange(fromProject(p));

	return (
		<div className="overflow-y-auto max-h-80">
			<OutputSection project={fakeProject} update={update} save={update} />
			<LayoutSection project={fakeProject} update={update} save={update} />
			<SpriteSection project={fakeProject} update={update} save={update} />
			<VariantSection project={fakeProject} update={update} save={update} />
		</div>
	);
}
