#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

pub mod app;
pub mod menu;
pub mod panels;
pub mod state;
pub mod toolbar;
pub mod widgets;
pub mod worker;

use std::path::PathBuf;

/// Launch the native GUI window.
///
/// `project_path` is the optional `.fpsheet` file to open on startup.
pub fn run(project_path: Option<PathBuf>) -> anyhow::Result<()> {
    let mut app = app::FastPackApp::default();
    if let Some(path) = project_path {
        match std::fs::read_to_string(&path) {
            Ok(text) => match toml::from_str(&text) {
                Ok(project) => {
                    app.state.project = project;
                    app.state.project_path = Some(path);
                }
                Err(e) => app.state.log_error(format!("Failed to parse project: {e}")),
            },
            Err(e) => app.state.log_error(format!("Failed to read project: {e}")),
        }
    }
    let options = eframe::NativeOptions::default();
    eframe::run_native("FastPack", options, Box::new(|_cc| Ok(Box::new(app))))
        .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}
