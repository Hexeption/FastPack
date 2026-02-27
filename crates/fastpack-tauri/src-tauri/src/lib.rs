//! Tauri GUI backend for FastPack.

pub mod commands;
pub mod preferences;
pub mod state;
pub mod updater;
pub mod worker;

use std::path::PathBuf;

/// Launch the Tauri GUI window.
///
/// `project_path` is the optional `.fpsheet` file to open on startup.
pub fn run(project_path: Option<PathBuf>) -> anyhow::Result<()> {
    let app_state = state::TauriState::new(project_path);

    tauri::Builder::default()
        .manage(std::sync::Mutex::new(app_state))
        .invoke_handler(tauri::generate_handler![
            commands::project::new_project,
            commands::project::open_project,
            commands::project::save_project,
            commands::project::get_project,
            commands::project::update_project,
            commands::project::add_source,
            commands::project::remove_source,
            commands::project::handle_drop,
            commands::pack::pack,
            commands::pack::start_watch,
            commands::pack::stop_watch,
            commands::dialogs::open_folder_dialog,
            commands::dialogs::open_file_dialog,
            commands::dialogs::save_file_dialog,
            commands::dialogs::open_config_folder,
            commands::preferences::get_preferences,
            commands::preferences::save_preferences,
            commands::updater::check_for_update,
            commands::updater::download_update,
            commands::updater::apply_update,
            commands::cli::install_cli,
            commands::cli::check_cli_installed,
        ])
        .setup(|_app| {
            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                let window = _app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("tauri error: {e}"))
}
