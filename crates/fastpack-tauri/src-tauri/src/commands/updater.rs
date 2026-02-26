use crate::updater::{ReleaseInfo, do_apply, do_check, do_download};

#[tauri::command]
pub fn check_for_update() -> Result<Option<ReleaseInfo>, String> {
    do_check()
}

#[tauri::command]
pub fn download_update(url: String) -> Result<String, String> {
    do_download(&url).map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn apply_update(path: String) -> Result<(), String> {
    do_apply(std::path::Path::new(&path))
}
