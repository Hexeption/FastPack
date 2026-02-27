use rfd::FileDialog;

/// Open the FastPack config folder in the system file browser.
#[tauri::command]
pub fn open_config_folder() -> Result<(), String> {
    let path = dirs::config_dir()
        .map(|d| d.join("FastPack"))
        .ok_or("could not determine config directory")?;
    let _ = std::fs::create_dir_all(&path);

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Open a folder picker dialog. Returns the selected path or `None`.
#[tauri::command]
pub fn open_folder_dialog(starting_path: Option<String>) -> Option<String> {
    let mut dialog = FileDialog::new();
    if let Some(p) = starting_path {
        dialog = dialog.set_directory(p);
    }
    dialog
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
