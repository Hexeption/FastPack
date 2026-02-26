use eframe::egui;
use egui_phosphor::regular as ph;
use rust_i18n::t;

use crate::state::AppState;

fn icon_button(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    ui.button(format!("{icon}  {label}"))
}

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        let export_label = if state.packing {
            t!("toolbar.exporting").to_string()
        } else {
            t!("toolbar.export").to_string()
        };
        if ui
            .add_enabled(
                !state.packing,
                egui::Button::new(format!("{}  {}", ph::EXPORT, export_label)),
            )
            .clicked()
        {
            state.pending.export = true;
        }

        if icon_button(ui, ph::FOLDER_PLUS, &t!("toolbar.add_sprites")).clicked() {
            state.pending.add_source = true;
        }

        let can_preview = state.selected_frames.len() >= 2 && !state.frames.is_empty();
        if ui
            .add_enabled(
                can_preview,
                egui::Button::new(format!("{}  Preview [P]", ph::PLAY)),
            )
            .clicked()
        {
            state.anim_preview.open = true;
            state.anim_preview.current_frame = 0;
            state.anim_preview.elapsed_secs = 0.0;
            state.anim_preview.playing = true;
            state.anim_preview.zoom = 1.0;
            state.anim_preview.pan = [0.0, 0.0];
        }

        let count = state.project.sources.len();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let src_label = if count == 1 {
                t!("toolbar.sources_single", count = count)
            } else {
                t!("toolbar.sources_plural", count = count)
            };
            ui.label(src_label);
            ui.separator();

            let theme_icon = if state.dark_mode { ph::SUN } else { ph::MOON };
            let theme_label = if state.dark_mode {
                t!("toolbar.light").to_string()
            } else {
                t!("toolbar.dark").to_string()
            };
            if ui
                .small_button(format!("{}  {theme_label}", theme_icon))
                .clicked()
            {
                state.dark_mode = !state.dark_mode;
            }
        });
    });
    ui.add_space(2.0);
}
