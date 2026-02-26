use rfd::FileDialog;

/// Open a folder picker dialog. Returns the selected path or `None`.
#[tauri::command]
pub fn open_folder_dialog() -> Option<String> {
    FileDialog::new()
        .pick_folder()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Open a file picker dialog. Returns the selected path or `None`.
#[tauri::command]
pub fn open_file_dialog() -> Option<String> {
    FileDialog::new()
        .add_filter("FastPack project", &["fpsheet"])
        .add_filter("All files", &["*"])
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Open a save dialog. Returns the chosen path or `None`.
#[tauri::command]
pub fn save_file_dialog(default_name: String) -> Option<String> {
    FileDialog::new()
        .set_file_name(&default_name)
        .add_filter("FastPack project", &["fpsheet"])
        .save_file()
        .map(|p| p.to_string_lossy().into_owned())
}
