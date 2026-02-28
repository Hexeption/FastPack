use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::{LogEntry, TauriState};
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

/// Preview pack — builds atlas in memory, updates UI, no disk writes.
fn pack_impl(app: &AppHandle) {
    let (project, already_packing) = {
        let state_ref = app.state::<Mutex<TauriState>>();
        let mut guard = state_ref.lock().unwrap();
        if guard.is_packing {
            return;
        }
        guard.is_packing = true;
        (guard.project.clone(), false)
    };
    let _ = already_packing;

    let _ = app.emit("pack:started", ());

    let app_clone = app.clone();

    std::thread::spawn(move || match worker::run_pack(&project) {
        Ok(output) => {
            let sheets: Vec<crate::state::SheetData> = output
                .sheets
                .iter()
                .map(TauriState::sheet_to_data)
                .collect();

            let summary = format!(
                "Packed {} sprites ({} aliases, {} overflow) into {} sheet(s).",
                output.sprite_count,
                output.alias_count,
                output.overflow_count,
                sheets.len(),
            );

            let state_ref = app_clone.state::<Mutex<TauriState>>();
            let mut guard = state_ref.lock().unwrap();
            guard.is_packing = false;
            guard.sheets = sheets.clone();
            guard.sprite_count = output.sprite_count;
            guard.alias_count = output.alias_count;
            guard.overflow_count = output.overflow_count;
            guard.log_info(&summary);
            let log = guard.log.clone();
            drop(guard);

            let _ = app_clone.emit(
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
        Err(e) => {
            let msg = e.to_string();
            let state_ref = app_clone.state::<Mutex<TauriState>>();
            let mut guard = state_ref.lock().unwrap();
            guard.is_packing = false;
            guard.log_error(&msg);
            drop(guard);
            let _ = app_clone.emit("pack:failed", PackFailedPayload { error: msg });
        }
    });
}

/// Publish — re-packs and writes all output files to the configured directory.
fn publish_impl(app: &AppHandle) {
    let (project, project_path, already_packing) = {
        let state_ref = app.state::<Mutex<TauriState>>();
        let mut guard = state_ref.lock().unwrap();
        if guard.is_packing {
            return;
        }
        guard.is_packing = true;
        (guard.project.clone(), guard.project_path.clone(), false)
    };
    let _ = already_packing;

    let _ = app.emit("publish:started", ());

    let app_clone = app.clone();

    std::thread::spawn(move || {
        let result = worker::run_pack(&project)
            .and_then(|output| worker::write_output(&output, &project, project_path.as_deref()));

        match result {
            Ok((file_count, dir)) => {
                let directory = dir.display().to_string();
                let state_ref = app_clone.state::<Mutex<TauriState>>();
                let mut guard = state_ref.lock().unwrap();
                guard.is_packing = false;
                guard.log_info(format!("Published {file_count} file(s) to {directory}."));
                let log = guard.log.clone();
                drop(guard);

                let _ = app_clone.emit(
                    "publish:finished",
                    PublishFinishedPayload {
                        file_count,
                        directory,
                        log,
                    },
                );
            }
            Err(e) => {
                let msg = e.to_string();
                let state_ref = app_clone.state::<Mutex<TauriState>>();
                let mut guard = state_ref.lock().unwrap();
                guard.is_packing = false;
                guard.log_error(&msg);
                drop(guard);
                let _ = app_clone.emit("publish:failed", PublishFailedPayload { error: msg });
            }
        }
    });
}

/// Pack sprites into an in-memory atlas for UI preview. No files are written.
#[tauri::command]
pub fn pack(_state: State<'_, Mutex<TauriState>>, app: AppHandle) -> Result<(), String> {
    pack_impl(&app);
    Ok(())
}

/// Pack sprites and write all output files (textures + data) to the configured directory.
#[tauri::command]
pub fn publish(_state: State<'_, Mutex<TauriState>>, app: AppHandle) -> Result<(), String> {
    publish_impl(&app);
    Ok(())
}

/// Start watching source directories for changes, auto-packing on each change.
#[tauri::command]
pub fn start_watch(state: State<'_, Mutex<TauriState>>, app: AppHandle) -> Result<(), String> {
    use notify_debouncer_mini::notify::RecursiveMode;

    let source_dirs: Vec<std::path::PathBuf> = {
        let guard = state.lock().unwrap();
        guard
            .project
            .sources
            .iter()
            .filter(|s| s.path.is_dir())
            .map(|s| s.path.clone())
            .collect()
    };

    let app_debounce = app.clone();
    let mut debouncer = notify_debouncer_mini::new_debouncer(
        std::time::Duration::from_millis(500),
        move |_result| {
            let app = app_debounce.clone();
            std::thread::spawn(move || pack_impl(&app));
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
    guard.watcher = Some(crate::state::WatcherHandle {
        _debouncer: debouncer,
        stop_tx,
    });
    guard.log_info("Watch mode started.");
    Ok(())
}

/// Stop watching source directories.
#[tauri::command]
pub fn stop_watch(state: State<'_, Mutex<TauriState>>) -> Result<(), String> {
    let mut guard = state.lock().unwrap();
    guard.watcher = None;
    guard.log_info("Watch mode stopped.");
    Ok(())
}
