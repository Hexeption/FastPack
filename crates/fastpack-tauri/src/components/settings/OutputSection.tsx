import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { openFolderDialog } from "../../lib/commands";
import type { OutputConfig, Project } from "../../types";
import { Row, Section } from "../Section";

/** Shared props for all settings section components. */
interface SectionProps {
	project: Project;
	update: (p: Project) => void;
	save: (p: Project) => void;
}

/** Output settings: atlas name, directory, texture/data format, pixel format, quality, and multipack toggle. */
export default function OutputSection({ project, update, save }: SectionProps) {
	const { t } = useTranslation();
	const out = project.output;

	const setOut = (patch: Partial<OutputConfig>) =>
		update({ ...project, output: { ...out, ...patch } });
	const saveOut = (patch: Partial<OutputConfig>) =>
		save({ ...project, output: { ...out, ...patch } });

	const handlePickDir = async () => {
		const p = await openFolderDialog();
		if (p) saveOut({ directory: p });
	};

	return (
		<Section title={t("output.sectionTitle")}>
			<Row label={t("output.name")}>
				<Input
					type="text"
					value={out.name}
					className="h-6 text-xs w-full"
					onChange={(e) => saveOut({ name: e.target.value })}
				/>
			</Row>
			<Row label={t("output.directory")}>
				<Input
					type="text"
					value={
						typeof out.directory === "string"
							? out.directory
							: String(out.directory)
					}
					className="h-6 text-xs flex-1 min-w-0"
					onChange={(e) => saveOut({ directory: e.target.value })}
				/>
				<Button
					variant="outline"
					size="icon-xs"
					onClick={handlePickDir}
					title={t("output.browse")}
				>
					…
				</Button>
			</Row>
			<Row label={t("output.format")}>
				<Select
					value={out.texture_format}
					onValueChange={(v) =>
						saveOut({ texture_format: v as typeof out.texture_format })
					}
				>
					<SelectTrigger className="h-6 text-xs w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="png">{t("output.png")}</SelectItem>
						<SelectItem value="jpeg">{t("output.jpeg")}</SelectItem>
						<SelectItem value="webp">{t("output.webp")}</SelectItem>
					</SelectContent>
				</Select>
			</Row>
			<Row label={t("output.pixelFormat")}>
				<Select
					value={out.pixel_format}
					onValueChange={(v) =>
						saveOut({ pixel_format: v as typeof out.pixel_format })
					}
				>
					<SelectTrigger className="h-6 text-xs w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="rgba8888">{t("output.rgba8888")}</SelectItem>
						<SelectItem value="rgb888">{t("output.rgb888")}</SelectItem>
						<SelectItem value="rgb565">{t("output.rgb565")}</SelectItem>
						<SelectItem value="rgba4444">{t("output.rgba4444")}</SelectItem>
						<SelectItem value="rgba5551">{t("output.rgba5551")}</SelectItem>
						<SelectItem value="alpha8">{t("output.alpha8")}</SelectItem>
					</SelectContent>
				</Select>
			</Row>
			<Row label={t("output.data")}>
				<Select
					value={out.data_format}
					onValueChange={(v) =>
						saveOut({ data_format: v as typeof out.data_format })
					}
				>
					<SelectTrigger className="h-6 text-xs w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="json_hash">{t("output.jsonHash")}</SelectItem>
						<SelectItem value="json_array">{t("output.jsonArray")}</SelectItem>
						<SelectItem value="phaser3">{t("output.phaser3")}</SelectItem>
						<SelectItem value="pixijs">{t("output.pixijs")}</SelectItem>
					</SelectContent>
				</Select>
			</Row>
			<Row label={t("output.quality")}>
				<Slider
					value={[out.quality]}
					min={1}
					max={100}
					step={1}
					className="flex-1"
					onValueChange={([v]) => saveOut({ quality: v })}
				/>
				<span className="text-xs tabular-nums text-muted-foreground w-7 text-right shrink-0">
					{out.quality}
				</span>
			</Row>
			<Row label={t("output.pathPrefix")}>
				<Input
					type="text"
					value={out.texture_path_prefix}
					className="h-6 text-xs w-full"
					placeholder={t("output.pathPrefixPlaceholder")}
					onChange={(e) => saveOut({ texture_path_prefix: e.target.value })}
				/>
			</Row>
			{out.data_format !== "phaser3" && (
				<Row label={t("output.premultiplyAlpha")}>
					<Switch
						checked={out.premultiply_alpha}
						onCheckedChange={(c) => saveOut({ premultiply_alpha: c })}
					/>
				</Row>
			)}
			<Row label={t("output.multipack")}>
				<Switch
					checked={out.multipack}
					onCheckedChange={(c) => setOut({ multipack: c })}
				/>
			</Row>
		</Section>
	);
}
