//! GUI front-end for FastPack built on egui/eframe.
rust_i18n::i18n!("locales");

pub mod app;
pub mod menu;
pub mod panels;
pub mod preferences;
pub mod state;
pub mod theme;
pub mod toolbar;
/// Auto-update checking and download logic.
pub mod updater;
/// Reusable egui widget components.
pub mod widgets;
/// Background pack worker thread and message types.
pub mod worker;

use std::path::PathBuf;

use eframe::egui;
use rust_i18n::t;

/// Launch the native GUI window.
///
/// `project_path` is the optional `.fpsheet` file to open on startup.
pub fn run(project_path: Option<PathBuf>) -> anyhow::Result<()> {
    let mut app = app::FastPackApp::default();
    rust_i18n::set_locale(app.prefs.language.code());
    if let Some(path) = project_path {
        match std::fs::read_to_string(&path) {
            Ok(text) => match toml::from_str(&text) {
                Ok(project) => {
                    app.state.project = project;
                    app.state.project_path = Some(path);
                }
                Err(e) => app
                    .state
                    .log_error(t!("log.parse_failed", err = e.to_string())),
            },
            Err(e) => app
                .state
                .log_error(t!("log.read_failed", err = e.to_string())),
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "FastPack",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx, app.state.dark_mode);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}
