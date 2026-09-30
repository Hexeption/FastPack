//! Tauri GUI backend for FastPack.

pub mod commands;
#[cfg(target_os = "macos")]
pub mod menu;
pub mod preferences;
pub mod state;
pub mod updater;
pub mod worker;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{Emitter, Manager};

use state::AppState;

/// Launch the Tauri GUI window.
///
/// `project_path` is the optional `.fpsheet` file to open on startup.
pub fn run(project_path: Option<PathBuf>) -> anyhow::Result<()> {
    let app_state = state::AppState::new(project_path);

    let app = tauri::Builder::default()
        .manage(Mutex::new(app_state))
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
            commands::pack::publish,
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
            let id = event.id().as_ref();
            if id == "new_window" {
                let _ = commands::window::create_new_window(app);
                return;
            }
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
                // Only the focused window should act on a menu command.
                let focused = app
                    .webview_windows()
                    .into_values()
                    .find(|w| w.is_focused().unwrap_or(false));
                let _ = match focused {
                    Some(win) => app.emit_to(win.label(), n, ()),
                    None => app.emit(n, ()),
                };
            }
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event
                && let Some(state) = window.try_state::<Mutex<AppState>>()
            {
                state.lock().unwrap().windows.remove(window.label());
            }
        })
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            {
                let m = menu::build(_app)?;
                _app.set_menu(m)?;
            }

            build_tray(_app)?;

            #[cfg(debug_assertions)]
            if let Some(window) = _app.get_webview_window("main") {
                window.open_devtools();
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("tauri error: {e}"))?;

    app.run(|_app_handle, _event| {
        // macOS convention: closing the last window keeps the app running, and
        // clicking the dock icon with no windows open creates a new one.
        #[cfg(target_os = "macos")]
        match _event {
            tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } => api.prevent_exit(),
            tauri::RunEvent::Reopen {
                has_visible_windows: false,
                ..
            } => {
                let _ = commands::window::create_new_window(_app_handle);
            }
            _ => {}
        }
    });

    Ok(())
}

/// System tray icon with "New Window" and "Quit" entries.
fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{MenuBuilder, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let new_win = MenuItem::with_id(app, "tray_new_window", "New Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray_quit", "Quit FastPack", true, None::<&str>)?;
    let menu = MenuBuilder::new(app)
        .item(&new_win)
        .separator()
        .item(&quit)
        .build()?;

    let mut tray = TrayIconBuilder::new()
        .tooltip("FastPack")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "tray_new_window" => {
                let _ = commands::window::create_new_window(app);
            }
            "tray_quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
