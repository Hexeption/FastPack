use std::sync::Mutex;

use tauri::State;

use crate::preferences::Preferences;
use crate::state::AppState;

#[tauri::command]
pub fn get_preferences(state: State<'_, Mutex<AppState>>) -> Preferences {
    state.lock().unwrap().prefs.clone()
}

#[tauri::command]
pub fn save_preferences(
    state: State<'_, Mutex<AppState>>,
    prefs: Preferences,
) -> Result<(), String> {
    prefs.save();
    state.lock().unwrap().prefs = prefs;
    Ok(())
}
