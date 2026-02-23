use eframe::egui;

use crate::state::AppState;

/// Left panel: scrollable list of sprite entries in the project.
pub fn show(ui: &mut egui::Ui, _state: &mut AppState) {
    ui.heading("Sprites");
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label("(no sprites)");
    });
}
