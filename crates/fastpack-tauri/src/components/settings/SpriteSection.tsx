import { useTranslation } from "react-i18next";
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
import type { Project, SpriteConfig } from "../../types";
import { Row, Section } from "../Section";

interface SectionProps {
	project: Project;
	update: (p: Project) => void;
	save: (p: Project) => void;
}

export default function SpriteSection({ project, update }: SectionProps) {
	const { t } = useTranslation();
	const sprites = project.sprites;

	const setSprites = (patch: Partial<SpriteConfig>) =>
		update({ ...project, sprites: { ...sprites, ...patch } });

	return (
		<Section title={t("sprites.sectionTitle")}>
			<Row label={t("sprites.trimMode")}>
				<Select
					value={sprites.trim_mode}
					onValueChange={(v) =>
						setSprites({ trim_mode: v as typeof sprites.trim_mode })
					}
				>
					<SelectTrigger className="h-6 text-xs">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="none">{t("sprites.trimNone")}</SelectItem>
						<SelectItem value="trim">{t("sprites.trimTrim")}</SelectItem>
						<SelectItem value="crop">{t("sprites.trimCrop")}</SelectItem>
						<SelectItem value="crop_keep_pos">
							{t("sprites.trimCropKeepPos")}
						</SelectItem>
						<SelectItem value="polygon">{t("sprites.trimPolygon")}</SelectItem>
					</SelectContent>
				</Select>
			</Row>
			<Row
				label={t("sprites.trimThreshold", { value: sprites.trim_threshold })}
			>
				<Slider
					value={[sprites.trim_threshold]}
					min={0}
					max={255}
					step={1}
					className="w-[120px]"
					onValueChange={([v]) => setSprites({ trim_threshold: v })}
				/>
			</Row>
			<Row label={t("sprites.trimMargin")}>
				<Input
					type="number"
					value={sprites.trim_margin}
					min={0}
					className="h-6 text-xs"
					onChange={(e) => setSprites({ trim_margin: Number(e.target.value) })}
				/>
			</Row>
			<Row label={t("sprites.extrude")}>
				<Input
					type="number"
					value={sprites.extrude}
					min={0}
					className="h-6 text-xs"
					onChange={(e) => setSprites({ extrude: Number(e.target.value) })}
				/>
			</Row>
			<Row label={t("sprites.divisorXY")}>
				<div className="flex items-center gap-1">
					<Input
						type="number"
						value={sprites.common_divisor_x}
						min={1}
						className="h-6 w-[55px] text-xs"
						onChange={(e) =>
							setSprites({ common_divisor_x: Number(e.target.value) })
						}
					/>
					<span className="text-xs text-muted-foreground">/</span>
					<Input
						type="number"
						value={sprites.common_divisor_y}
						min={1}
						className="h-6 w-[55px] text-xs"
						onChange={(e) =>
							setSprites({ common_divisor_y: Number(e.target.value) })
						}
					/>
				</div>
			</Row>
			<Row label={t("sprites.pivotXY")}>
				<div className="flex items-center gap-1">
					<Input
						type="number"
						value={sprites.default_pivot.x}
						min={0}
						max={1}
						step={0.1}
						className="h-6 w-[55px] text-xs"
						onChange={(e) =>
							setSprites({
								default_pivot: {
									...sprites.default_pivot,
									x: Number(e.target.value),
								},
							})
						}
					/>
					<span className="text-xs text-muted-foreground">/</span>
					<Input
						type="number"
						value={sprites.default_pivot.y}
						min={0}
						max={1}
						step={0.1}
						className="h-6 w-[55px] text-xs"
						onChange={(e) =>
							setSprites({
								default_pivot: {
									...sprites.default_pivot,
									y: Number(e.target.value),
								},
							})
						}
					/>
				</div>
			</Row>
			<Row label={t("sprites.detectAliases")}>
				<Switch
					checked={sprites.detect_aliases}
					onCheckedChange={(c) => setSprites({ detect_aliases: c })}
				/>
			</Row>
		</Section>
	);
}
