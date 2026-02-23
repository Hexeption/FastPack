use eframe::egui;

use crate::state::{AppState, LogLevel};

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.strong("Output");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("Clear").clicked() {
                state.log.clear();
            }
        });
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for entry in &state.log {
                let color = match entry.level {
                    LogLevel::Info => egui::Color32::from_gray(220),
                    LogLevel::Warn => egui::Color32::from_rgb(255, 200, 60),
                    LogLevel::Error => egui::Color32::from_rgb(255, 90, 80),
                };
                let text = format!("[{}] {}", entry.time, entry.message);
                ui.label(egui::RichText::new(text).color(color).small().monospace());
            }
        });
}
