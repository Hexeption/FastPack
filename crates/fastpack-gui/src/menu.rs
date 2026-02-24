use eframe::egui;
use rust_i18n::t;

use crate::state::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button(t!("menu.file"), |ui| {
            if ui
                .add(egui::Button::new(t!("menu.new")).shortcut_text("Ctrl+N"))
                .clicked()
            {
                state.pending.new_project = true;
                ui.close_menu();
            }
            if ui
                .add(egui::Button::new(t!("menu.open")).shortcut_text("Ctrl+O"))
                .clicked()
            {
                state.pending.open_project = true;
                ui.close_menu();
            }
            ui.separator();
            if ui
                .add(egui::Button::new(t!("menu.save")).shortcut_text("Ctrl+S"))
                .clicked()
            {
                state.pending.save_project = true;
                ui.close_menu();
            }
            if ui.button(t!("menu.save_as")).clicked() {
                state.pending.save_project_as = true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button(t!("menu.quit")).clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ui.menu_button(t!("menu.edit"), |ui| {
            if ui.button(t!("menu.preferences")).clicked() {
                state.pending.open_prefs = true;
                ui.close_menu();
            }
        });

        ui.menu_button(t!("menu.atlas"), |ui| {
            let label = if state.packing {
                t!("menu.exporting")
            } else {
                t!("menu.export")
            };
            if ui
                .add_enabled(
                    !state.packing,
                    egui::Button::new(label).shortcut_text("Ctrl+P"),
                )
                .clicked()
            {
                state.pending.export = true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button(t!("menu.add_sprites")).clicked() {
                state.pending.add_source = true;
                ui.close_menu();
            }
        });

        // Keyboard shortcuts
        let ctx = ui.ctx().clone();
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::N) {
                state.pending.new_project = true;
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::O) {
                state.pending.open_project = true;
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::S) {
                state.pending.save_project = true;
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::P) && !state.packing {
                state.pending.export = true;
            }
            if i.consume_key(egui::Modifiers::NONE, egui::Key::P)
                && state.selected_frames.len() >= 2
                && !state.frames.is_empty()
            {
                state.anim_preview.open = !state.anim_preview.open;
                if state.anim_preview.open {
                    state.anim_preview.current_frame = 0;
                    state.anim_preview.elapsed_secs = 0.0;
                    state.anim_preview.playing = true;
                    state.anim_preview.zoom = 1.0;
                    state.anim_preview.pan = [0.0, 0.0];
                }
            }
        });
    });
}
