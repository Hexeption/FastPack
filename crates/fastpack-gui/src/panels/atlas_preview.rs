use eframe::egui;

use crate::state::AppState;

pub fn show(ui: &mut egui::Ui, _state: &mut AppState, _atlas: Option<&egui::TextureHandle>) {
    ui.centered_and_justified(|ui| {
        ui.label("Atlas preview will appear here after packing.");
    });
}
