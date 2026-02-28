import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import {
	InputGroup,
	InputGroupAddon,
	InputGroupInput,
} from "@/components/ui/input-group";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type { AlgorithmConfig, LayoutConfig, Project } from "../../types";
import { Row, Section } from "../Section";

/** Shared props for all settings section components. */
interface SectionProps {
	project: Project;
	update: (p: Project) => void;
	save: (p: Project) => void;
}

/** Layout settings: max dimensions, algorithm, heuristic, padding, rotation, and size constraints. */
export default function LayoutSection({ project, update }: SectionProps) {
	const { t } = useTranslation();
	const layout = project.layout;
	const alg = project.algorithm;

	const setLayout = (patch: Partial<LayoutConfig>) =>
		update({ ...project, layout: { ...layout, ...patch } });

	const algType = (() => {
		if (alg.type === "grid") return "grid";
		if (alg.type === "basic") return "basic";
		if (alg.type === "max_rects") return "max_rects";
		return "polygon";
	})();

	const setAlgType = (v: string) => {
		let newAlg: AlgorithmConfig;
		if (v === "grid") newAlg = { type: "grid", cell_width: 0, cell_height: 0 };
		else if (v === "basic") newAlg = { type: "basic" };
		else if (v === "polygon") newAlg = { type: "polygon" };
		else newAlg = { type: "max_rects", heuristic: "best_short_side_fit" };
		update({ ...project, algorithm: newAlg });
	};

	return (
		<Section title={t("layout.sectionTitle")}>
			<Row label={t("layout.maxSize")}>
				<InputGroup className="h-6 flex-1">
					<InputGroupAddon className="px-1.5 py-0 text-[10px]">
						W
					</InputGroupAddon>
					<InputGroupInput
						type="number"
						value={layout.max_width}
						min={1}
						className="text-xs h-full py-0"
						onChange={(e) => setLayout({ max_width: Number(e.target.value) })}
					/>
				</InputGroup>
				<InputGroup className="h-6 flex-1">
					<InputGroupAddon className="px-1.5 py-0 text-[10px]">
						H
					</InputGroupAddon>
					<InputGroupInput
						type="number"
						value={layout.max_height}
						min={1}
						className="text-xs h-full py-0"
						onChange={(e) => setLayout({ max_height: Number(e.target.value) })}
					/>
				</InputGroup>
			</Row>
			<Row label={t("layout.fixedWidth")}>
				<Switch
					checked={layout.fixed_width !== null}
					onCheckedChange={(c) =>
						setLayout({ fixed_width: c ? layout.max_width : null })
					}
				/>
				<Input
					type="number"
					value={layout.fixed_width ?? layout.max_width}
					min={1}
					disabled={layout.fixed_width === null}
					className="h-6 text-xs w-[60px] disabled:opacity-40"
					onChange={(e) => setLayout({ fixed_width: Number(e.target.value) })}
				/>
			</Row>
			<Row label={t("layout.fixedHeight")}>
				<Switch
					checked={layout.fixed_height !== null}
					onCheckedChange={(c) =>
						setLayout({ fixed_height: c ? layout.max_height : null })
					}
				/>
				<Input
					type="number"
					value={layout.fixed_height ?? layout.max_height}
					min={1}
					disabled={layout.fixed_height === null}
					className="h-6 text-xs w-[60px] disabled:opacity-40"
					onChange={(e) => setLayout({ fixed_height: Number(e.target.value) })}
				/>
			</Row>
			<Row label={t("layout.algorithm")}>
				<Select value={algType} onValueChange={(v) => setAlgType(v)}>
					<SelectTrigger className="h-6 text-xs w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="max_rects">{t("layout.maxRects")}</SelectItem>
						<SelectItem value="grid">{t("layout.grid")}</SelectItem>
						<SelectItem value="basic">{t("layout.basic")}</SelectItem>
						<SelectItem value="polygon">{t("layout.polygon")}</SelectItem>
					</SelectContent>
				</Select>
			</Row>
			{alg.type === "max_rects" && (
				<Row label={t("layout.heuristic")}>
					<Select
						value={alg.heuristic}
						onValueChange={(v) =>
							update({
								...project,
								algorithm: {
									type: "max_rects",
									heuristic: v as typeof alg.heuristic,
								},
							})
						}
					>
						<SelectTrigger className="h-6 text-xs w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="best_short_side_fit">
								{t("layout.bestShortSide")}
							</SelectItem>
							<SelectItem value="best_long_side_fit">
								{t("layout.bestLongSide")}
							</SelectItem>
							<SelectItem value="best_area_fit">
								{t("layout.bestArea")}
							</SelectItem>
							<SelectItem value="bottom_left_rule">
								{t("layout.bottomLeft")}
							</SelectItem>
							<SelectItem value="contact_point_rule">
								{t("layout.contactPoint")}
							</SelectItem>
						</SelectContent>
					</Select>
				</Row>
			)}
			{alg.type === "grid" && (
				<Row label={t("layout.cellSize")}>
					<InputGroup className="h-6 flex-1">
						<InputGroupAddon className="px-1.5 py-0 text-[10px]">
							W
						</InputGroupAddon>
						<InputGroupInput
							type="number"
							value={alg.cell_width}
							min={1}
							className="text-xs h-full py-0"
							onChange={(e) =>
								update({
									...project,
									algorithm: {
										type: "grid",
										cell_width: Number(e.target.value),
										cell_height: alg.cell_height,
									},
								})
							}
						/>
					</InputGroup>
					<InputGroup className="h-6 flex-1">
						<InputGroupAddon className="px-1.5 py-0 text-[10px]">
							H
						</InputGroupAddon>
						<InputGroupInput
							type="number"
							value={alg.cell_height}
							min={1}
							className="text-xs h-full py-0"
							onChange={(e) =>
								update({
									...project,
									algorithm: {
										type: "grid",
										cell_width: alg.cell_width,
										cell_height: Number(e.target.value),
									},
								})
							}
						/>
					</InputGroup>
				</Row>
			)}
			<Row label={t("layout.packMode")}>
				<Select
					value={layout.pack_mode}
					onValueChange={(v) =>
						setLayout({ pack_mode: v as typeof layout.pack_mode })
					}
				>
					<SelectTrigger className="h-6 text-xs w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="fast">{t("layout.fast")}</SelectItem>
						<SelectItem value="good">{t("layout.good")}</SelectItem>
						<SelectItem value="best">{t("layout.best")}</SelectItem>
					</SelectContent>
				</Select>
			</Row>
			<Row label={t("layout.sizeConstraint")}>
				<Select
					value={layout.size_constraint}
					onValueChange={(v) =>
						setLayout({
							size_constraint: v as typeof layout.size_constraint,
						})
					}
				>
					<SelectTrigger className="h-6 text-xs w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="any_size">{t("layout.any")}</SelectItem>
						<SelectItem value="pot">{t("layout.powerOf2")}</SelectItem>
						<SelectItem value="multiple_of4">
							{t("layout.multipleOf4")}
						</SelectItem>
						<SelectItem value="word_aligned">
							{t("layout.wordAligned")}
						</SelectItem>
					</SelectContent>
				</Select>
			</Row>
			<Row label={t("layout.borderPad")}>
				<Input
					type="number"
					value={layout.border_padding}
					min={0}
					className="h-6 text-xs w-full"
					onChange={(e) =>
						setLayout({ border_padding: Number(e.target.value) })
					}
				/>
			</Row>
			<Row label={t("layout.shapePad")}>
				<Input
					type="number"
					value={layout.shape_padding}
					min={0}
					className="h-6 text-xs w-full"
					onChange={(e) => setLayout({ shape_padding: Number(e.target.value) })}
				/>
			</Row>
			<Row label={t("layout.rotation")}>
				<Switch
					checked={layout.allow_rotation}
					onCheckedChange={(c) => setLayout({ allow_rotation: c })}
				/>
			</Row>
			<Row label={t("layout.forceSquare")}>
				<Switch
					checked={layout.force_square}
					onCheckedChange={(c) => setLayout({ force_square: c })}
				/>
			</Row>
		</Section>
	);
}
