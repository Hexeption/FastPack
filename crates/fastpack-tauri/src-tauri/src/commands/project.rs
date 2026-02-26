use std::sync::Mutex;

use fastpack_core::types::config::{Project, SourceSpec};
use tauri::State;

use crate::state::TauriState;

#[tauri::command]
pub fn new_project(state: State<'_, Mutex<TauriState>>) -> Project {
    let mut s = state.lock().unwrap();
    let default_config = s.prefs.default_config.clone();
    s.project = Project::default();
    s.project.config = default_config;
    s.project_path = None;
    s.dirty = false;
    s.sheets.clear();
    s.log_info("New project created.");
    s.project.clone()
}

#[tauri::command]
pub fn open_project(state: State<'_, Mutex<TauriState>>, path: String) -> Result<Project, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let project: Project = toml::from_str(&text).map_err(|e| e.to_string())?;
    let mut s = state.lock().unwrap();
    s.project = project.clone();
    s.project_path = Some(std::path::PathBuf::from(&path));
    s.dirty = false;
    s.sheets.clear();
    s.log_info(format!("Opened: {path}"));
    Ok(project)
}

#[tauri::command]
pub fn save_project(state: State<'_, Mutex<TauriState>>, path: String) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    let text = toml::to_string_pretty(&s.project).map_err(|e| e.to_string())?;
    std::fs::write(&path, text.as_bytes()).map_err(|e| e.to_string())?;
    s.project_path = Some(std::path::PathBuf::from(&path));
    s.dirty = false;
    s.log_info(format!("Saved: {path}"));
    Ok(())
}

#[tauri::command]
pub fn get_project(state: State<'_, Mutex<TauriState>>) -> Project {
    state.lock().unwrap().project.clone()
}

#[tauri::command]
pub fn update_project(state: State<'_, Mutex<TauriState>>, project: Project) {
    let mut s = state.lock().unwrap();
    s.project = project;
    s.dirty = true;
}

#[tauri::command]
pub fn add_source(state: State<'_, Mutex<TauriState>>, path: String) -> Project {
    let mut s = state.lock().unwrap();
    let pb = std::path::PathBuf::from(&path);
    let canonical = std::fs::canonicalize(&pb).unwrap_or(pb);

    let already_tracked = s.project.sources.iter().any(|src| {
        let stored = std::fs::canonicalize(&src.path).unwrap_or_else(|_| src.path.clone());
        canonical.starts_with(&stored)
    });

    if !already_tracked {
        let display = canonical.display().to_string();
        s.project.sources.push(SourceSpec {
            path: canonical,
            filter: "**/*.png".to_string(),
        });
        s.dirty = true;
        s.log_info(format!("Added source: {display}"));
    }

    s.project.clone()
}

#[tauri::command]
pub fn remove_source(state: State<'_, Mutex<TauriState>>, index: usize) -> Project {
    let mut s = state.lock().unwrap();
    if index < s.project.sources.len() {
        let removed = s.project.sources.remove(index);
        s.dirty = true;
        s.log_info(format!("Removed source: {}", removed.path.display()));
    }
    s.project.clone()
}
