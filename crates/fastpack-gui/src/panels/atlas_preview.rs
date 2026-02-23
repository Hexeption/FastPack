use eframe::egui;

use crate::state::AppState;

/// Central panel: zoomable atlas texture preview.
pub fn show(ui: &mut egui::Ui, _state: &mut AppState) {
    ui.centered_and_justified(|ui| {
        ui.label("Atlas preview will appear here after packing.");
    });
}
