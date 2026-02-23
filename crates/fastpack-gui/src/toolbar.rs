use eframe::egui;

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

        let count = state.project.sources.len();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!(
                "{count} source{}",
                if count == 1 { "" } else { "s" }
            ));
            ui.separator();
            let theme_label = if state.dark_mode { "Light" } else { "Dark" };
            if ui.small_button(theme_label).clicked() {
                state.dark_mode = !state.dark_mode;
            }
        });
    });
}
