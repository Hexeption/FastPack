use eframe::egui;

use crate::state::AppState;

/// Bottom panel: scrollable log of pack output and errors.
pub fn show(ui: &mut egui::Ui, _state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.heading("Output");
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.label("Ready.");
        });
}
