use std::sync::Mutex;

use tauri::State;

use crate::preferences::Preferences;
use crate::state::TauriState;

#[tauri::command]
pub fn get_preferences(state: State<'_, Mutex<TauriState>>) -> Preferences {
    state.lock().unwrap().prefs.clone()
}

#[tauri::command]
pub fn save_preferences(
    state: State<'_, Mutex<TauriState>>,
    prefs: Preferences,
) -> Result<(), String> {
    prefs.save();
    state.lock().unwrap().prefs = prefs;
    Ok(())
}
