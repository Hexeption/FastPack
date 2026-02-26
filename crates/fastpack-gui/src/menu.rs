use eframe::egui;
use rust_i18n::t;

use crate::{
    preferences::{KeyBind, Keybinds},
    state::AppState,
};

pub fn show(ui: &mut egui::Ui, state: &mut AppState, keybinds: &Keybinds) {
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button(t!("menu.file"), |ui| {
            if ui
                .add(
                    egui::Button::new(t!("menu.new")).shortcut_text(keybinds.new_project.display()),
                )
                .clicked()
            {
                state.pending.new_project = true;
                ui.close();
            }
            if ui
                .add(
                    egui::Button::new(t!("menu.open"))
                        .shortcut_text(keybinds.open_project.display()),
                )
                .clicked()
            {
                state.pending.open_project = true;
                ui.close();
            }
            ui.separator();
            if ui
                .add(
                    egui::Button::new(t!("menu.save"))
                        .shortcut_text(keybinds.save_project.display()),
                )
                .clicked()
            {
                state.pending.save_project = true;
                ui.close();
            }
            if ui.button(t!("menu.save_as")).clicked() {
                state.pending.save_project_as = true;
                ui.close();
            }
            ui.separator();
            if ui.button(t!("menu.quit")).clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ui.menu_button(t!("menu.edit"), |ui| {
            if ui.button(t!("menu.preferences")).clicked() {
                state.pending.open_prefs = true;
                ui.close();
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
                    egui::Button::new(label).shortcut_text(keybinds.export.display()),
                )
                .clicked()
            {
                state.pending.export = true;
                ui.close();
            }
            ui.separator();
            if ui.button(t!("menu.add_sprites")).clicked() {
                state.pending.add_source = true;
                ui.close();
            }
        });

        // Keyboard shortcuts
        let ctx = ui.ctx().clone();
        let is_capturing =
            ctx.data(|d| d.get_temp::<u8>(egui::Id::new("kb_capturing")).unwrap_or(0) != 0);

        ctx.input_mut(|i| {
            if !is_capturing {
                if let Some(key) = str_to_egui_key(&keybinds.new_project.key) {
                    if i.consume_key(bind_to_modifiers(&keybinds.new_project), key) {
                        state.pending.new_project = true;
                    }
                }
                if let Some(key) = str_to_egui_key(&keybinds.open_project.key) {
                    if i.consume_key(bind_to_modifiers(&keybinds.open_project), key) {
                        state.pending.open_project = true;
                    }
                }
                if let Some(key) = str_to_egui_key(&keybinds.save_project.key) {
                    if i.consume_key(bind_to_modifiers(&keybinds.save_project), key) {
                        state.pending.save_project = true;
                    }
                }
                if let Some(key) = str_to_egui_key(&keybinds.export.key) {
                    if i.consume_key(bind_to_modifiers(&keybinds.export), key) && !state.packing {
                        state.pending.export = true;
                    }
                }
                if let Some(key) = str_to_egui_key(&keybinds.anim_preview.key) {
                    if i.consume_key(bind_to_modifiers(&keybinds.anim_preview), key)
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
                }
            }
        });
    });
}

fn bind_to_modifiers(bind: &KeyBind) -> egui::Modifiers {
    egui::Modifiers {
        alt: bind.alt,
        ctrl: bind.ctrl,
        shift: bind.shift,
        mac_cmd: false,
        command: bind.ctrl,
    }
}

pub(crate) fn str_to_egui_key(s: &str) -> Option<egui::Key> {
    match s {
        "A" => Some(egui::Key::A),
        "B" => Some(egui::Key::B),
        "C" => Some(egui::Key::C),
        "D" => Some(egui::Key::D),
        "E" => Some(egui::Key::E),
        "F" => Some(egui::Key::F),
        "G" => Some(egui::Key::G),
        "H" => Some(egui::Key::H),
        "I" => Some(egui::Key::I),
        "J" => Some(egui::Key::J),
        "K" => Some(egui::Key::K),
        "L" => Some(egui::Key::L),
        "M" => Some(egui::Key::M),
        "N" => Some(egui::Key::N),
        "O" => Some(egui::Key::O),
        "P" => Some(egui::Key::P),
        "Q" => Some(egui::Key::Q),
        "R" => Some(egui::Key::R),
        "S" => Some(egui::Key::S),
        "T" => Some(egui::Key::T),
        "U" => Some(egui::Key::U),
        "V" => Some(egui::Key::V),
        "W" => Some(egui::Key::W),
        "X" => Some(egui::Key::X),
        "Y" => Some(egui::Key::Y),
        "Z" => Some(egui::Key::Z),
        "0" => Some(egui::Key::Num0),
        "1" => Some(egui::Key::Num1),
        "2" => Some(egui::Key::Num2),
        "3" => Some(egui::Key::Num3),
        "4" => Some(egui::Key::Num4),
        "5" => Some(egui::Key::Num5),
        "6" => Some(egui::Key::Num6),
        "7" => Some(egui::Key::Num7),
        "8" => Some(egui::Key::Num8),
        "9" => Some(egui::Key::Num9),
        "F1" => Some(egui::Key::F1),
        "F2" => Some(egui::Key::F2),
        "F3" => Some(egui::Key::F3),
        "F4" => Some(egui::Key::F4),
        "F5" => Some(egui::Key::F5),
        "F6" => Some(egui::Key::F6),
        "F7" => Some(egui::Key::F7),
        "F8" => Some(egui::Key::F8),
        "F9" => Some(egui::Key::F9),
        "F10" => Some(egui::Key::F10),
        "F11" => Some(egui::Key::F11),
        "F12" => Some(egui::Key::F12),
        "Enter" => Some(egui::Key::Enter),
        "Space" => Some(egui::Key::Space),
        "Tab" => Some(egui::Key::Tab),
        "Delete" => Some(egui::Key::Delete),
        "Backspace" => Some(egui::Key::Backspace),
        "Home" => Some(egui::Key::Home),
        "End" => Some(egui::Key::End),
        "Insert" => Some(egui::Key::Insert),
        _ => None,
    }
}

pub(crate) fn egui_key_to_str(key: egui::Key) -> Option<&'static str> {
    match key {
        egui::Key::A => Some("A"),
        egui::Key::B => Some("B"),
        egui::Key::C => Some("C"),
        egui::Key::D => Some("D"),
        egui::Key::E => Some("E"),
        egui::Key::F => Some("F"),
        egui::Key::G => Some("G"),
        egui::Key::H => Some("H"),
        egui::Key::I => Some("I"),
        egui::Key::J => Some("J"),
        egui::Key::K => Some("K"),
        egui::Key::L => Some("L"),
        egui::Key::M => Some("M"),
        egui::Key::N => Some("N"),
        egui::Key::O => Some("O"),
        egui::Key::P => Some("P"),
        egui::Key::Q => Some("Q"),
        egui::Key::R => Some("R"),
        egui::Key::S => Some("S"),
        egui::Key::T => Some("T"),
        egui::Key::U => Some("U"),
        egui::Key::V => Some("V"),
        egui::Key::W => Some("W"),
        egui::Key::X => Some("X"),
        egui::Key::Y => Some("Y"),
        egui::Key::Z => Some("Z"),
        egui::Key::Num0 => Some("0"),
        egui::Key::Num1 => Some("1"),
        egui::Key::Num2 => Some("2"),
        egui::Key::Num3 => Some("3"),
        egui::Key::Num4 => Some("4"),
        egui::Key::Num5 => Some("5"),
        egui::Key::Num6 => Some("6"),
        egui::Key::Num7 => Some("7"),
        egui::Key::Num8 => Some("8"),
        egui::Key::Num9 => Some("9"),
        egui::Key::F1 => Some("F1"),
        egui::Key::F2 => Some("F2"),
        egui::Key::F3 => Some("F3"),
        egui::Key::F4 => Some("F4"),
        egui::Key::F5 => Some("F5"),
        egui::Key::F6 => Some("F6"),
        egui::Key::F7 => Some("F7"),
        egui::Key::F8 => Some("F8"),
        egui::Key::F9 => Some("F9"),
        egui::Key::F10 => Some("F10"),
        egui::Key::F11 => Some("F11"),
        egui::Key::F12 => Some("F12"),
        egui::Key::Enter => Some("Enter"),
        egui::Key::Space => Some("Space"),
        egui::Key::Tab => Some("Tab"),
        egui::Key::Delete => Some("Delete"),
        egui::Key::Backspace => Some("Backspace"),
        egui::Key::Home => Some("Home"),
        egui::Key::End => Some("End"),
        egui::Key::Insert => Some("Insert"),
        _ => None,
    }
}
