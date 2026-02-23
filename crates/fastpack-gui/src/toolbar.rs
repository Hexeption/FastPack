use eframe::egui;
use fastpack_core::types::config::{AlgorithmConfig, MaxRectsHeuristic, PackMode};

use crate::state::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        let pack_label = if state.packing { "Packing…" } else { "Pack" };
        if ui
            .add_enabled(!state.packing, egui::Button::new(pack_label))
            .clicked()
        {
            state.pending.pack = true;
        }

        if ui.button("Add Sprites…").clicked() {
            state.pending.add_source = true;
        }

        ui.separator();

        let algo_label = match &state.project.config.algorithm {
            AlgorithmConfig::Grid { .. } => "Grid",
            AlgorithmConfig::Basic => "Basic",
            AlgorithmConfig::MaxRects { .. } => "MaxRects",
            AlgorithmConfig::Polygon => "Polygon",
        };
        let mut new_algo: Option<AlgorithmConfig> = None;
        egui::ComboBox::from_id_salt("algo_picker")
            .selected_text(algo_label)
            .width(90.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(
                        matches!(state.project.config.algorithm, AlgorithmConfig::Grid { .. }),
                        "Grid",
                    )
                    .clicked()
                {
                    new_algo = Some(AlgorithmConfig::Grid {
                        cell_width: 0,
                        cell_height: 0,
                    });
                }
                if ui
                    .selectable_label(
                        matches!(state.project.config.algorithm, AlgorithmConfig::Basic),
                        "Basic",
                    )
                    .clicked()
                {
                    new_algo = Some(AlgorithmConfig::Basic);
                }
                if ui
                    .selectable_label(
                        matches!(
                            state.project.config.algorithm,
                            AlgorithmConfig::MaxRects { .. }
                        ),
                        "MaxRects",
                    )
                    .clicked()
                {
                    new_algo = Some(AlgorithmConfig::MaxRects {
                        heuristic: MaxRectsHeuristic::BestShortSideFit,
                    });
                }
                if ui
                    .selectable_label(
                        matches!(state.project.config.algorithm, AlgorithmConfig::Polygon),
                        "Polygon",
                    )
                    .clicked()
                {
                    new_algo = Some(AlgorithmConfig::Polygon);
                }
            });
        if let Some(algo) = new_algo {
            state.project.config.algorithm = algo;
            state.dirty = true;
        }

        let cur_mode = state.project.config.layout.pack_mode;
        let mut new_mode = cur_mode;
        egui::ComboBox::from_id_salt("pack_mode_picker")
            .selected_text(match cur_mode {
                PackMode::Fast => "Fast",
                PackMode::Good => "Good",
                PackMode::Best => "Best",
            })
            .width(70.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut new_mode, PackMode::Fast, "Fast");
                ui.selectable_value(&mut new_mode, PackMode::Good, "Good");
                ui.selectable_value(&mut new_mode, PackMode::Best, "Best");
            });
        if new_mode != cur_mode {
            state.project.config.layout.pack_mode = new_mode;
            state.dirty = true;
        }

        let count = state.project.sources.len();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!(
                "{count} source{}",
                if count == 1 { "" } else { "s" }
            ));
        });
    });
}
