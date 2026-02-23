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
    let mut state = state::AppState::default();
    state.project_path = project_path;

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "FastPack",
        options,
        Box::new(|_cc| Ok(Box::new(app::FastPackApp { state }))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}
