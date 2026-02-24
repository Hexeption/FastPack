use eframe::egui;

use crate::state::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        let export_label = if state.packing {
            "Exporting..."
        } else {
            "Export"
        };
        if ui
            .add_enabled(!state.packing, egui::Button::new(export_label))
            .clicked()
        {
            state.pending.export = true;
        }

        if ui.button("Add Sprites…").clicked() {
            state.pending.add_source = true;
        }

        let count = state.project.sources.len();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!(
                "{count} source{}",
                if count == 1 { "" } else { "s" }
            ));
            ui.separator();

            let sheet_count = state.sheets.len();
            if sheet_count > 1 {
                let cur = state.current_sheet;
                if ui.small_button("▶").clicked() && cur + 1 < sheet_count {
                    state.current_sheet += 1;
                    state.frames = state.sheets[state.current_sheet].frames.clone();
                    state.selected_frame = None;
                }
                ui.label(format!("Sheet {} of {}", cur + 1, sheet_count));
                if ui.small_button("◀").clicked() && cur > 0 {
                    state.current_sheet -= 1;
                    state.frames = state.sheets[state.current_sheet].frames.clone();
                    state.selected_frame = None;
                }
                ui.separator();
            }

            let theme_label = if state.dark_mode { "Light" } else { "Dark" };
            if ui.small_button(theme_label).clicked() {
                state.dark_mode = !state.dark_mode;
            }
        });
    });
    ui.add_space(2.0);
}
