use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui;
use egui_phosphor::regular as ph;
use rust_i18n::t;

use crate::{
    panels::settings,
    preferences::{KeyBind, Keybinds, Language, Preferences},
    state::AppState,
    updater::{self, UpdateMsg, UpdateStatus},
};

#[derive(Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    General,
    Defaults,
    Updates,
    Keybinds,
}

/// Render the preferences window and poll pending update messages.
pub fn show(
    ctx: &egui::Context,
    prefs: &mut Preferences,
    open: &mut bool,
    update_status: &mut UpdateStatus,
    update_rx: &mut Option<mpsc::Receiver<UpdateMsg>>,
) {
    poll_updates(update_status, update_rx);

    egui::Window::new(t!("prefs.title"))
        .open(open)
        .resizable(true)
        .default_size([520.0, 500.0])
        .collapsible(false)
        .show(ctx, |ui| {
            let tab_id = egui::Id::new("prefs_active_tab");
            let mut tab: Tab = ctx.data(|d| d.get_temp(tab_id).unwrap_or_default());

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut tab,
                    Tab::General,
                    format!("{}  {}", ph::GEAR, t!("prefs.tab_general")),
                );
                ui.selectable_value(
                    &mut tab,
                    Tab::Defaults,
                    format!("{}  {}", ph::SLIDERS, t!("prefs.tab_defaults")),
                );
                ui.selectable_value(
                    &mut tab,
                    Tab::Updates,
                    format!("{}  {}", ph::CLOUD_ARROW_DOWN, t!("prefs.tab_updates")),
                );
                ui.selectable_value(
                    &mut tab,
                    Tab::Keybinds,
                    format!("{}  Keybinds", ph::KEYBOARD),
                );
            });
            ui.separator();

            ctx.data_mut(|d| d.insert_temp(tab_id, tab));

            match tab {
                Tab::General => show_general(ui, prefs),
                Tab::Defaults => show_defaults(ui, prefs),
                Tab::Updates => show_updates(ui, prefs, update_status, update_rx),
                Tab::Keybinds => show_keybinds(ui, prefs),
            }
        });
}

fn poll_updates(
    update_status: &mut UpdateStatus,
    update_rx: &mut Option<mpsc::Receiver<UpdateMsg>>,
) {
    let Some(rx) = update_rx.as_ref() else { return };
    match rx.try_recv() {
        Ok(UpdateMsg::UpToDate { latest }) => {
            *update_status = UpdateStatus::UpToDate { latest };
            *update_rx = None;
        }
        Ok(UpdateMsg::Available(info)) => {
            *update_status = UpdateStatus::Available(info);
            *update_rx = None;
        }
        Ok(UpdateMsg::Downloaded(path)) => {
            *update_status = UpdateStatus::Downloaded(path);
            *update_rx = None;
        }
        Ok(UpdateMsg::Error(msg)) => {
            *update_status = UpdateStatus::Error(msg);
            *update_rx = None;
        }
        Err(mpsc::TryRecvError::Empty) => {}
        Err(mpsc::TryRecvError::Disconnected) => {
            *update_rx = None;
        }
    }
}

fn show_general(ui: &mut egui::Ui, prefs: &mut Preferences) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(t!("prefs.language"));
        let mut selected = prefs.language;
        egui::ComboBox::from_id_salt("language_selector")
            .selected_text(selected.display())
            .show_ui(ui, |ui| {
                for &lang in Language::ALL {
                    ui.selectable_value(&mut selected, lang, lang.display());
                }
            });
        if selected != prefs.language {
            prefs.language = selected;
            rust_i18n::set_locale(selected.code());
            prefs.save();
        }
    });

    const SCALE_STEPS: &[(f32, &str)] = &[
        (0.75, "75%"),
        (1.0, "100%"),
        (1.25, "125%"),
        (1.5, "150%"),
        (1.75, "175%"),
        (2.0, "200%"),
    ];

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("UI Scale");
        let current_label = SCALE_STEPS
            .iter()
            .find(|&&(v, _)| (v - prefs.ui_scale).abs() < 0.01)
            .map(|&(_, s)| s)
            .unwrap_or("Custom");
        let mut changed = false;
        egui::ComboBox::from_id_salt("ui_scale_selector")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for &(value, label) in SCALE_STEPS {
                    if ui
                        .selectable_label((prefs.ui_scale - value).abs() < 0.01, label)
                        .clicked()
                    {
                        prefs.ui_scale = value;
                        changed = true;
                    }
                }
            });
        if changed {
            prefs.save();
        }
    });
}

fn show_defaults(ui: &mut egui::Ui, prefs: &mut Preferences) {
    let mut tmp = AppState::default();
    tmp.project.config = prefs.default_config.clone();

    egui::ScrollArea::vertical()
        .id_salt("prefs_defaults_scroll")
        .show(ui, |ui| {
            section(ui, "texture", t!("settings.texture"), |ui| {
                settings::show_texture(ui, &mut tmp)
            });
            section(ui, "layout", t!("settings.layout"), |ui| {
                settings::show_layout(ui, &mut tmp)
            });
            section(ui, "sprites", t!("settings.sprites"), |ui| {
                settings::show_sprites(ui, &mut tmp)
            });
            section(ui, "variants", t!("settings.variants"), |ui| {
                settings::show_variants(ui, &mut tmp)
            });

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(t!("prefs.defaults_hint"))
                    .small()
                    .weak(),
            );
        });

    if tmp.dirty {
        prefs.default_config = tmp.project.config;
        prefs.save();
    }
}

fn section(
    ui: &mut egui::Ui,
    id_key: &str,
    label: impl Into<String>,
    body: impl FnOnce(&mut egui::Ui),
) {
    let id = egui::Id::new(("prefs_section", id_key));
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            ui.strong(label.into());
        })
        .body(|ui| {
            ui.add_space(2.0);
            body(ui);
            ui.add_space(4.0);
        });
    ui.separator();
}

fn show_updates(
    ui: &mut egui::Ui,
    prefs: &mut Preferences,
    update_status: &mut UpdateStatus,
    update_rx: &mut Option<mpsc::Receiver<UpdateMsg>>,
) {
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(t!("prefs.version"));
        ui.strong(format!("v{}", updater::CURRENT_VERSION));
    });

    ui.horizontal(|ui| {
        ui.label(t!("prefs.latest"));
        match update_status {
            UpdateStatus::Idle => {
                ui.weak(t!("prefs.not_checked"));
            }
            UpdateStatus::Checking => {
                ui.weak(t!("prefs.checking"));
            }
            UpdateStatus::UpToDate { latest } => {
                ui.label(format!("v{}  {}", latest, t!("prefs.up_to_date")));
            }
            UpdateStatus::Available(info) => {
                ui.strong(format!(
                    "v{} {}",
                    info.version,
                    t!("prefs.update_available")
                ));
            }
            UpdateStatus::Downloading => {
                ui.weak(t!("prefs.downloading"));
            }
            UpdateStatus::Downloaded(_) => {
                ui.label(t!("prefs.downloaded"));
            }
            UpdateStatus::Error(msg) => {
                ui.colored_label(egui::Color32::from_rgb(220, 70, 70), msg.as_str());
            }
        }
    });

    ui.add_space(8.0);

    let busy = matches!(
        update_status,
        UpdateStatus::Checking | UpdateStatus::Downloading
    );
    if ui
        .add_enabled(!busy, egui::Button::new(t!("prefs.check_updates")))
        .clicked()
    {
        let (tx, rx) = mpsc::channel();
        updater::spawn_check(tx);
        *update_rx = Some(rx);
        *update_status = UpdateStatus::Checking;
    }

    if ui
        .checkbox(&mut prefs.auto_check_updates, t!("prefs.auto_check"))
        .changed()
    {
        prefs.save();
    }

    ui.add_space(8.0);

    if let UpdateStatus::Available(info) = update_status {
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("prefs_notes_scroll")
                .max_height(150.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(&info.notes).small())
                            .wrap_mode(egui::TextWrapMode::Wrap),
                    );
                });
        });
        ui.add_space(4.0);
        let info_clone = info.clone();
        if ui
            .add_enabled(!busy, egui::Button::new(t!("prefs.download_install")))
            .clicked()
        {
            let (tx, rx) = mpsc::channel();
            updater::spawn_download(info_clone, tx);
            *update_rx = Some(rx);
            *update_status = UpdateStatus::Downloading;
        }
    }

    if let UpdateStatus::Downloaded(path) = update_status {
        ui.label(t!("prefs.restart_hint"));
        let path: PathBuf = path.clone();
        if ui.button(t!("prefs.restart")).clicked() {
            if let Err(e) = updater::apply_update(&path) {
                *update_status = UpdateStatus::Error(e);
            }
        }
    }
}

fn show_keybinds(ui: &mut egui::Ui, prefs: &mut Preferences) {
    let cap_id = egui::Id::new("kb_capturing");
    let mut cap: u8 = ui.ctx().data(|d| d.get_temp(cap_id).unwrap_or(0));
    let mut save = false;

    if cap != 0 {
        let mut captured: Option<KeyBind> = None;
        let mut canceled = false;
        ui.ctx().input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    if *key == egui::Key::Escape {
                        canceled = true;
                    } else if let Some(name) = crate::menu::egui_key_to_str(*key) {
                        captured = Some(KeyBind {
                            key: name.to_owned(),
                            ctrl: modifiers.ctrl,
                            shift: modifiers.shift,
                            alt: modifiers.alt,
                        });
                    }
                }
            }
        });
        if canceled {
            cap = 0;
        }
        if let Some(bind) = captured {
            match cap {
                1 => prefs.keybinds.new_project = bind,
                2 => prefs.keybinds.open_project = bind,
                3 => prefs.keybinds.save_project = bind,
                4 => prefs.keybinds.export = bind,
                5 => prefs.keybinds.anim_preview = bind,
                _ => {}
            }
            cap = 0;
            save = true;
        }
        ui.ctx().request_repaint();
    }

    ui.ctx().data_mut(|d| d.insert_temp(cap_id, cap));

    ui.add_space(4.0);
    if keybind_row(ui, "New Project", &prefs.keybinds.new_project, cap == 1) {
        cap = 1;
    }
    if keybind_row(ui, "Open Project", &prefs.keybinds.open_project, cap == 2) {
        cap = 2;
    }
    if keybind_row(ui, "Save Project", &prefs.keybinds.save_project, cap == 3) {
        cap = 3;
    }
    if keybind_row(ui, "Export", &prefs.keybinds.export, cap == 4) {
        cap = 4;
    }
    if keybind_row(
        ui,
        "Animation Preview",
        &prefs.keybinds.anim_preview,
        cap == 5,
    ) {
        cap = 5;
    }

    ui.ctx().data_mut(|d| d.insert_temp(cap_id, cap));

    ui.add_space(8.0);
    if ui.button("Reset to defaults").clicked() {
        prefs.keybinds = Keybinds::default();
        save = true;
    }

    if save {
        prefs.save();
    }
}

fn keybind_row(ui: &mut egui::Ui, label: &str, bind: &KeyBind, capturing: bool) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if capturing {
                ui.label(egui::RichText::new("Press any key...").weak().italics());
            } else {
                if ui.small_button("Change").clicked() {
                    clicked = true;
                }
                ui.label(bind.display());
            }
        });
    });
    ui.separator();
    clicked
}
