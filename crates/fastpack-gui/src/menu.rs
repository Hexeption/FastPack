use eframe::egui;

use crate::state::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui
                .add(egui::Button::new("New").shortcut_text("Ctrl+N"))
                .clicked()
            {
                state.pending.new_project = true;
                ui.close_menu();
            }
            if ui
                .add(egui::Button::new("Open…").shortcut_text("Ctrl+O"))
                .clicked()
            {
                state.pending.open_project = true;
                ui.close_menu();
            }
            ui.separator();
            if ui
                .add(egui::Button::new("Save").shortcut_text("Ctrl+S"))
                .clicked()
            {
                state.pending.save_project = true;
                ui.close_menu();
            }
            if ui.button("Save As…").clicked() {
                state.pending.save_project_as = true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Quit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ui.menu_button("Pack", |ui| {
            let label = if state.packing {
                "Packing…"
            } else {
                "Pack Now"
            };
            if ui
                .add_enabled(
                    !state.packing,
                    egui::Button::new(label).shortcut_text("Ctrl+P"),
                )
                .clicked()
            {
                state.pending.pack = true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Add Sprites…").clicked() {
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
                state.pending.pack = true;
            }
        });
    });
}
