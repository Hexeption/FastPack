use eframe::egui;

use crate::state::AppState;

/// Render the toolbar row (Pack, Add, Remove buttons).
pub fn show(ui: &mut egui::Ui, _state: &mut AppState) {
    ui.horizontal(|ui| {
        if ui.button("Pack").clicked() {}
        ui.separator();
        if ui.button("Add Sprites…").clicked() {}
        if ui.button("Remove").clicked() {}
    });
}
