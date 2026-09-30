use std::collections::BTreeSet;
use std::sync::Mutex;

use fastpack_core::types::config::{Project, SourceSpec};
use tauri::{State, WebviewWindow};

use crate::state::AppState;

#[tauri::command]
pub fn new_project(state: State<'_, Mutex<AppState>>, webview_window: WebviewWindow) -> Project {
    let mut s = state.lock().unwrap();
    let label = webview_window.label();
    let default_config = s.prefs.default_config.clone();
    let w = s.window_mut(label);
    w.project = Project::default();
    w.project.config = default_config;
    w.project_path = None;
    w.dirty = false;
    w.sheets.clear();
    w.log_info("New project created.");
    w.project.clone()
}

#[tauri::command]
pub fn open_project(
    state: State<'_, Mutex<AppState>>,
    webview_window: WebviewWindow,
    path: String,
) -> Result<Project, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let project: Project = toml::from_str(&text).map_err(|e| e.to_string())?;
    let mut s = state.lock().unwrap();
    let w = s.window_mut(webview_window.label());
    w.project = project.clone();
    w.project_path = Some(std::path::PathBuf::from(&path));
    w.dirty = false;
    w.sheets.clear();
    w.log_info(format!("Opened: {path}"));
    Ok(project)
}

#[tauri::command]
pub fn save_project(
    state: State<'_, Mutex<AppState>>,
    webview_window: WebviewWindow,
    path: String,
) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    let w = s.window_mut(webview_window.label());
    let text = toml::to_string_pretty(&w.project).map_err(|e| e.to_string())?;
    std::fs::write(&path, text.as_bytes()).map_err(|e| e.to_string())?;
    w.project_path = Some(std::path::PathBuf::from(&path));
    w.dirty = false;
    w.log_info(format!("Saved: {path}"));
    Ok(())
}

#[tauri::command]
pub fn get_project(state: State<'_, Mutex<AppState>>, webview_window: WebviewWindow) -> Project {
    state
        .lock()
        .unwrap()
        .window_mut(webview_window.label())
        .project
        .clone()
}

#[tauri::command]
pub fn update_project(
    state: State<'_, Mutex<AppState>>,
    webview_window: WebviewWindow,
    project: Project,
) {
    let mut s = state.lock().unwrap();
    let w = s.window_mut(webview_window.label());
    w.project = project;
    w.dirty = true;
}

#[tauri::command]
pub fn add_source(
    state: State<'_, Mutex<AppState>>,
    webview_window: WebviewWindow,
    path: String,
) -> Project {
    let mut s = state.lock().unwrap();
    let w = s.window_mut(webview_window.label());
    let pb = std::path::PathBuf::from(&path);
    let canonical = std::fs::canonicalize(&pb).unwrap_or(pb);

    let already_tracked = w.project.sources.iter().any(|src| {
        let stored = std::fs::canonicalize(&src.path).unwrap_or_else(|_| src.path.clone());
        canonical.starts_with(&stored)
    });

    if !already_tracked {
        let display = canonical.display().to_string();
        w.project.sources.push(SourceSpec {
            path: canonical,
            filter: "**/*.png".to_string(),
        });
        w.dirty = true;
        w.log_info(format!("Added source: {display}"));
    }

    w.project.clone()
}

#[tauri::command]
pub fn remove_source(
    state: State<'_, Mutex<AppState>>,
    webview_window: WebviewWindow,
    index: usize,
) -> Project {
    let mut s = state.lock().unwrap();
    let w = s.window_mut(webview_window.label());
    if index < w.project.sources.len() {
        let removed = w.project.sources.remove(index);
        w.dirty = true;
        w.log_info(format!("Removed source: {}", removed.path.display()));
    }
    w.project.clone()
}

#[derive(serde::Serialize)]
pub struct HandleDropResult {
    pub project: Project,
    pub project_path: Option<String>,
    pub dirty: bool,
}

#[tauri::command]
pub fn handle_drop(
    state: State<'_, Mutex<AppState>>,
    webview_window: WebviewWindow,
    paths: Vec<String>,
) -> Result<HandleDropResult, String> {
    let mut s = state.lock().unwrap();
    let w = s.window_mut(webview_window.label());
    let mut new_sources: BTreeSet<std::path::PathBuf> = BTreeSet::new();

    for raw in &paths {
        let pb = std::path::PathBuf::from(raw);

        // .fpsheet → open the project
        if pb.extension().and_then(|e| e.to_str()) == Some("fpsheet") {
            let text = std::fs::read_to_string(&pb).map_err(|e| e.to_string())?;
            let project: Project = toml::from_str(&text).map_err(|e| e.to_string())?;
            let canon = std::fs::canonicalize(&pb).unwrap_or(pb);
            w.project = project;
            w.project_path = Some(canon.clone());
            w.dirty = false;
            w.sheets.clear();
            w.log_info(format!("Opened: {}", canon.display()));
            return Ok(HandleDropResult {
                project: w.project.clone(),
                project_path: Some(canon.display().to_string()),
                dirty: false,
            });
        }

        // directory → add directly; file → add parent
        let candidate = if pb.is_dir() {
            std::fs::canonicalize(&pb).unwrap_or_else(|_| pb.clone())
        } else if let Some(parent) = pb.parent() {
            std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf())
        } else {
            continue;
        };

        new_sources.insert(candidate);
    }

    // batch dedup: drop /a/b if /a is also in the set
    let all: Vec<_> = new_sources.iter().cloned().collect();
    let deduped: Vec<_> = all
        .iter()
        .filter(|p| !all.iter().any(|other| other != *p && p.starts_with(other)))
        .cloned()
        .collect();

    for path in deduped {
        let already = w.project.sources.iter().any(|src| {
            let stored = std::fs::canonicalize(&src.path).unwrap_or_else(|_| src.path.clone());
            path.starts_with(&stored)
        });
        if !already {
            let display = path.display().to_string();
            w.project.sources.push(SourceSpec {
                path,
                filter: "**/*.png".to_string(),
            });
            w.dirty = true;
            w.log_info(format!("Added source: {display}"));
        }
    }

    let project_path = w.project_path.as_ref().map(|p| p.display().to_string());

    Ok(HandleDropResult {
        project: w.project.clone(),
        project_path,
        dirty: w.dirty,
    })
}
