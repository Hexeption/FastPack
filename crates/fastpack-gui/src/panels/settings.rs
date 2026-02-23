use eframe::egui;

use crate::state::AppState;

/// Right panel: packer settings form.
pub fn show(ui: &mut egui::Ui, _state: &mut AppState) {
    ui.heading("Settings");
    ui.separator();
    ui.label("Packer settings will be shown here.");
}
