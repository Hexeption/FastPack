use eframe::egui;

use crate::state::AppState;

/// Render the top menu bar (File, Pack).
pub fn show(ui: &mut egui::Ui, _state: &mut AppState) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New").clicked() {}
            if ui.button("Open…").clicked() {}
            if ui.button("Save").clicked() {}
            if ui.button("Save As…").clicked() {}
            ui.separator();
            if ui.button("Quit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ui.menu_button("Pack", |ui| {
            if ui.button("Pack Now").clicked() {}
            if ui.button("Watch Mode").clicked() {}
        });
    });
}
