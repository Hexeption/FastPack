//! Tauri GUI backend for FastPack.

pub mod commands;
#[cfg(target_os = "macos")]
pub mod menu;
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
        .on_menu_event(|app, event| {
            use tauri::Emitter;
            let id = event.id().as_ref();
            let name = match id {
                "new_project" => Some("menu:new-project"),
                "open_project" => Some("menu:open-project"),
                "save_project" => Some("menu:save"),
                "save_project_as" => Some("menu:save-as"),
                "toggle_theme" => Some("menu:toggle-theme"),
                "preferences" => Some("menu:preferences"),
                _ => None,
            };
            if let Some(n) = name {
                let _ = app.emit(n, ());
            }
        })
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            {
                let m = menu::build(_app)?;
                _app.set_menu(m)?;
            }

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
