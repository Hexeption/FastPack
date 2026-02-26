use eframe::egui;
use egui_phosphor::regular as ph;
use rust_i18n::t;

use crate::state::{AppState, LogLevel};

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.strong(t!("output_log.title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(ph::TRASH).clicked() {
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
                let (icon, color) = match entry.level {
                    LogLevel::Info => (ph::INFO, egui::Color32::from_gray(200)),
                    LogLevel::Warn => (ph::WARNING, egui::Color32::from_rgb(255, 200, 60)),
                    LogLevel::Error => (ph::X_CIRCLE, egui::Color32::from_rgb(255, 90, 80)),
                };
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).color(color).small());
                    ui.label(
                        egui::RichText::new(format!("[{}] {}", entry.time, entry.message))
                            .color(color)
                            .small()
                            .monospace(),
                    );
                });
            }
        });
}
