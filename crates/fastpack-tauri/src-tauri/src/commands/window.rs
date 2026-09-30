use std::sync::Mutex;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::state::AppState;

/// Open a new FastPack window with its own blank project.
///
/// Called from the app menu, the tray menu, and the macOS dock reopen event.
pub fn create_new_window(app: &AppHandle) -> tauri::Result<()> {
    let label = {
        let state = app.state::<Mutex<AppState>>();
        let mut s = state.lock().unwrap();
        let label = s.next_label();
        // Register state before the webview loads so its first commands find it.
        s.window_mut(&label);
        label
    };

    let built = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(Default::default()))
        .title("FastPack")
        .inner_size(1280.0, 800.0)
        .min_inner_size(900.0, 600.0)
        .build();

    if built.is_err() {
        let state = app.state::<Mutex<AppState>>();
        state.lock().unwrap().windows.remove(&label);
    }
    built.map(|_| ())
}
