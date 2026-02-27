import { Plus, X } from "lucide-react";
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
import type { Project, ScaleVariant } from "../../types";
import { Section } from "../Section";

interface SectionProps {
	project: Project;
	update: (p: Project) => void;
	save: (p: Project) => void;
}

export default function VariantSection({ project, update }: SectionProps) {
	const { t } = useTranslation();

	const addVariant = () => {
		const v: ScaleVariant = { scale: 1, suffix: "@1x", scale_mode: "smooth" };
		update({ ...project, variants: [...project.variants, v] });
	};

	const removeVariant = (i: number) => {
		const variants = project.variants.filter((_, idx) => idx !== i);
		update({ ...project, variants });
	};

	const setVariant = (i: number, patch: Partial<ScaleVariant>) => {
		const variants = project.variants.map((v, idx) =>
			idx === i ? { ...v, ...patch } : v,
		);
		update({ ...project, variants });
	};

	return (
		<Section title={t("variants.sectionTitle")} defaultOpen={false}>
			<div className="space-y-2">
				{project.variants.map((v, i) => (
					<div
						key={i}
						className="flex items-center gap-1 p-1.5 rounded-md border border-border bg-background"
					>
						<Input
							type="number"
							value={v.scale}
							min={0.1}
							step={0.5}
							className="h-6 w-[50px] text-xs"
							title={t("variants.scale")}
							onChange={(e) => setVariant(i, { scale: Number(e.target.value) })}
						/>
						<Input
							type="text"
							value={v.suffix}
							className="h-6 w-[55px] text-xs"
							title={t("variants.suffix")}
							placeholder="@1x"
							onChange={(e) => setVariant(i, { suffix: e.target.value })}
						/>
						<Select
							value={v.scale_mode}
							onValueChange={(val) =>
								setVariant(i, {
									scale_mode: val as ScaleVariant["scale_mode"],
								})
							}
						>
							<SelectTrigger className="h-6 text-xs">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value="smooth">{t("variants.smooth")}</SelectItem>
								<SelectItem value="fast">{t("variants.fast")}</SelectItem>
								<SelectItem value="scale2x">{t("variants.scale2x")}</SelectItem>
								<SelectItem value="scale3x">{t("variants.scale3x")}</SelectItem>
								<SelectItem value="hq2x">{t("variants.hq2x")}</SelectItem>
								<SelectItem value="eagle">{t("variants.eagle")}</SelectItem>
							</SelectContent>
						</Select>
						<Button
							variant="ghost"
							size="icon-xs"
							className="shrink-0 ml-auto"
							onClick={() => removeVariant(i)}
						>
							<X className="size-3" />
						</Button>
					</div>
				))}
				<Button
					variant="outline"
					size="xs"
					className="w-full"
					onClick={addVariant}
				>
					<Plus className="size-3" /> {t("variants.addVariant")}
				</Button>
			</div>
		</Section>
	);
}
