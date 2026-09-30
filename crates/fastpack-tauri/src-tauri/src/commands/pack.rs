use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

use crate::state::{AppState, LogEntry};
use crate::worker;

/// Payload emitted on `pack:finished` with atlas metadata for the UI.
#[derive(serde::Serialize, Clone)]
struct PackFinishedPayload {
    sprite_count: usize,
    alias_count: usize,
    overflow_count: usize,
    sheets: Vec<crate::state::SheetData>,
    log: Vec<LogEntry>,
}

/// Payload emitted on `pack:failed` with the error message.
#[derive(serde::Serialize, Clone)]
struct PackFailedPayload {
    error: String,
}

/// Payload emitted on `publish:finished` with write summary.
#[derive(serde::Serialize, Clone)]
struct PublishFinishedPayload {
    file_count: usize,
    directory: String,
    log: Vec<LogEntry>,
}

/// Payload emitted on `publish:failed` with the error message.
#[derive(serde::Serialize, Clone)]
struct PublishFailedPayload {
    error: String,
}

/// Emit an event to the window with `label` only.
///
/// `Emitter::emit` broadcasts to every window, even when called on a window
/// handle, so per-window events must go through `emit_to`.
fn emit_to(app: &AppHandle, label: &str, event: &str, payload: impl serde::Serialize + Clone) {
    let _ = app.emit_to(label, event, payload);
}

/// Preview pack — builds atlas in memory, updates UI, no disk writes.
fn pack_impl(app: &AppHandle, window_label: String) {
    let project = {
        let state_ref = app.state::<Mutex<AppState>>();
        let mut guard = state_ref.lock().unwrap();
        let Some(w) = guard.windows.get_mut(&window_label) else {
            return;
        };
        if w.is_packing {
            return;
        }
        w.is_packing = true;
        w.project.clone()
    };

    emit_to(app, &window_label, "pack:started", ());

    let app_clone = app.clone();
    let label = window_label.clone();

    std::thread::spawn(move || match worker::run_pack(&project) {
        Ok(output) => {
            let sheets: Vec<crate::state::SheetData> = output
                .sheets
                .iter()
                .map(crate::state::sheet_to_data)
                .collect();

            let summary = format!(
                "Packed {} sprites ({} aliases, {} overflow) into {} sheet(s).",
                output.sprite_count,
                output.alias_count,
                output.overflow_count,
                sheets.len(),
            );

            let state_ref = app_clone.state::<Mutex<AppState>>();
            let mut guard = state_ref.lock().unwrap();
            if let Some(w) = guard.windows.get_mut(&label) {
                w.is_packing = false;
                w.sheets = sheets.clone();
                w.sprite_count = output.sprite_count;
                w.alias_count = output.alias_count;
                w.overflow_count = output.overflow_count;
                w.log_info(&summary);
                let log = w.log.clone();
                drop(guard);

                emit_to(
                    &app_clone,
                    &label,
                    "pack:finished",
                    PackFinishedPayload {
                        sprite_count: output.sprite_count,
                        alias_count: output.alias_count,
                        overflow_count: output.overflow_count,
                        sheets,
                        log,
                    },
                );
            }
        }
        Err(e) => {
            let msg = e.to_string();
            let state_ref = app_clone.state::<Mutex<AppState>>();
            let mut guard = state_ref.lock().unwrap();
            if let Some(w) = guard.windows.get_mut(&label) {
                w.is_packing = false;
                w.log_error(&msg);
            }
            drop(guard);
            emit_to(
                &app_clone,
                &label,
                "pack:failed",
                PackFailedPayload { error: msg },
            );
        }
    });
}

/// Publish — re-packs and writes all output files to the configured directory.
fn publish_impl(app: &AppHandle, window_label: String) {
    let (project, project_path) = {
        let state_ref = app.state::<Mutex<AppState>>();
        let mut guard = state_ref.lock().unwrap();
        let Some(w) = guard.windows.get_mut(&window_label) else {
            return;
        };
        if w.is_packing {
            return;
        }
        w.is_packing = true;
        (w.project.clone(), w.project_path.clone())
    };

    emit_to(app, &window_label, "publish:started", ());

    let app_clone = app.clone();
    let label = window_label.clone();

    std::thread::spawn(move || {
        let result = worker::run_pack(&project)
            .and_then(|output| worker::write_output(&output, &project, project_path.as_deref()));

        match result {
            Ok((file_count, dir)) => {
                let directory = dir.display().to_string();
                let state_ref = app_clone.state::<Mutex<AppState>>();
                let mut guard = state_ref.lock().unwrap();
                if let Some(w) = guard.windows.get_mut(&label) {
                    w.is_packing = false;
                    w.log_info(format!("Published {file_count} file(s) to {directory}."));
                    let log = w.log.clone();
                    drop(guard);

                    emit_to(
                        &app_clone,
                        &label,
                        "publish:finished",
                        PublishFinishedPayload {
                            file_count,
                            directory,
                            log,
                        },
                    );
                }
            }
            Err(e) => {
                let msg = e.to_string();
                let state_ref = app_clone.state::<Mutex<AppState>>();
                let mut guard = state_ref.lock().unwrap();
                if let Some(w) = guard.windows.get_mut(&label) {
                    w.is_packing = false;
                    w.log_error(&msg);
                }
                drop(guard);
                emit_to(
                    &app_clone,
                    &label,
                    "publish:failed",
                    PublishFailedPayload { error: msg },
                );
            }
        }
    });
}

/// Pack sprites into an in-memory atlas for UI preview. No files are written.
#[tauri::command]
pub fn pack(
    _state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    webview_window: WebviewWindow,
) -> Result<(), String> {
    pack_impl(&app, webview_window.label().to_string());
    Ok(())
}

/// Pack sprites and write all output files (textures + data) to the configured directory.
#[tauri::command]
pub fn publish(
    _state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    webview_window: WebviewWindow,
) -> Result<(), String> {
    publish_impl(&app, webview_window.label().to_string());
    Ok(())
}

/// Start watching source directories for changes, auto-packing on each change.
#[tauri::command]
pub fn start_watch(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    webview_window: WebviewWindow,
) -> Result<(), String> {
    use notify_debouncer_mini::notify::RecursiveMode;

    let label = webview_window.label().to_string();

    let source_dirs: Vec<std::path::PathBuf> = {
        let mut guard = state.lock().unwrap();
        guard
            .window_mut(&label)
            .project
            .sources
            .iter()
            .filter(|s| s.path.is_dir())
            .map(|s| s.path.clone())
            .collect()
    };

    let app_debounce = app.clone();
    let label_debounce = label.clone();
    let mut debouncer = notify_debouncer_mini::new_debouncer(
        std::time::Duration::from_millis(500),
        move |_result| {
            let app = app_debounce.clone();
            let l = label_debounce.clone();
            std::thread::spawn(move || pack_impl(&app, l));
        },
    )
    .map_err(|e| e.to_string())?;

    for dir in &source_dirs {
        debouncer
            .watcher()
            .watch(dir, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;
    }

    let (stop_tx, _stop_rx) = std::sync::mpsc::sync_channel(1);

    let mut guard = state.lock().unwrap();
    let w = guard.window_mut(&label);
    w.watcher = Some(crate::state::WatcherHandle {
        _debouncer: debouncer,
        stop_tx,
    });
    w.log_info("Watch mode started.");
    Ok(())
}

/// Stop watching source directories.
#[tauri::command]
pub fn stop_watch(
    state: State<'_, Mutex<AppState>>,
    webview_window: WebviewWindow,
) -> Result<(), String> {
    let mut guard = state.lock().unwrap();
    let w = guard.window_mut(webview_window.label());
    w.watcher = None;
    w.log_info("Watch mode stopped.");
    Ok(())
}
