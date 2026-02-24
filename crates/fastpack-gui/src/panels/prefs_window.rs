use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui;

use crate::{
    panels::settings,
    preferences::Preferences,
    state::AppState,
    updater::{self, UpdateMsg, UpdateStatus},
};

#[derive(Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    Updates,
    Defaults,
}

pub fn show(
    ctx: &egui::Context,
    prefs: &mut Preferences,
    open: &mut bool,
    update_status: &mut UpdateStatus,
    update_rx: &mut Option<mpsc::Receiver<UpdateMsg>>,
) {
    poll_updates(update_status, update_rx);

    egui::Window::new("Preferences")
        .open(open)
        .resizable(true)
        .default_size([520.0, 500.0])
        .collapsible(false)
        .show(ctx, |ui| {
            let tab_id = egui::Id::new("prefs_active_tab");
            let mut tab: Tab = ctx.data(|d| d.get_temp(tab_id).unwrap_or_default());

            ui.horizontal(|ui| {
                ui.selectable_value(&mut tab, Tab::Updates, "Updates");
                ui.selectable_value(&mut tab, Tab::Defaults, "Defaults");
            });
            ui.separator();

            ctx.data_mut(|d| d.insert_temp(tab_id, tab));

            match tab {
                Tab::Defaults => show_defaults(ui, prefs),
                Tab::Updates => show_updates(ui, prefs, update_status, update_rx),
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

fn show_defaults(ui: &mut egui::Ui, prefs: &mut Preferences) {
    let mut tmp = AppState::default();
    tmp.project.config = prefs.default_config.clone();

    egui::ScrollArea::vertical()
        .id_salt("prefs_defaults_scroll")
        .show(ui, |ui| {
            section(ui, "Texture", |ui| settings::show_texture(ui, &mut tmp));
            section(ui, "Layout", |ui| settings::show_layout(ui, &mut tmp));
            section(ui, "Sprites", |ui| settings::show_sprites(ui, &mut tmp));
            section(ui, "Variants", |ui| settings::show_variants(ui, &mut tmp));

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    "Changes are saved automatically and applied to every new project.",
                )
                .small()
                .weak(),
            );
        });

    if tmp.dirty {
        prefs.default_config = tmp.project.config;
        prefs.save();
    }
}

fn section(ui: &mut egui::Ui, label: &str, body: impl FnOnce(&mut egui::Ui)) {
    let id = egui::Id::new(("prefs_section", label));
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            ui.strong(label);
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
        ui.label("FastPack");
        ui.strong(format!("v{}", updater::CURRENT_VERSION));
    });

    ui.horizontal(|ui| {
        ui.label("Latest:");
        match update_status {
            UpdateStatus::Idle => {
                ui.weak("not checked");
            }
            UpdateStatus::Checking => {
                ui.weak("checking…");
            }
            UpdateStatus::UpToDate { latest } => {
                ui.label(format!("v{latest}  (up to date)"));
            }
            UpdateStatus::Available(info) => {
                ui.strong(format!("v{} — update available", info.version));
            }
            UpdateStatus::Downloading => {
                ui.weak("downloading…");
            }
            UpdateStatus::Downloaded(_) => {
                ui.label("downloaded, ready to install");
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
        .add_enabled(!busy, egui::Button::new("Check for Updates"))
        .clicked()
    {
        let (tx, rx) = mpsc::channel();
        updater::spawn_check(tx);
        *update_rx = Some(rx);
        *update_status = UpdateStatus::Checking;
    }

    if ui
        .checkbox(
            &mut prefs.auto_check_updates,
            "Check automatically on startup",
        )
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
            .add_enabled(!busy, egui::Button::new("Download and Install"))
            .clicked()
        {
            let (tx, rx) = mpsc::channel();
            updater::spawn_download(info_clone, tx);
            *update_rx = Some(rx);
            *update_status = UpdateStatus::Downloading;
        }
    }

    if let UpdateStatus::Downloaded(path) = update_status {
        ui.label("Download complete. The app will restart after applying the update.");
        let path: PathBuf = path.clone();
        if ui.button("Restart and Update").clicked() {
            if let Err(e) = updater::apply_update(&path) {
                *update_status = UpdateStatus::Error(e);
            }
        }
    }
}
