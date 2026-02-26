use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::{LogEntry, TauriState};
use crate::worker;

/// Payload emitted with "pack:finished".
#[derive(serde::Serialize, Clone)]
struct PackFinishedPayload {
    sprite_count: usize,
    alias_count: usize,
    overflow_count: usize,
    sheets: Vec<crate::state::SheetData>,
    log: Vec<LogEntry>,
}

/// Payload emitted with "pack:failed".
#[derive(serde::Serialize, Clone)]
struct PackFailedPayload {
    error: String,
}

/// Run a full pack and emit events on the app handle.
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

    let project_clone = project.clone();
    let app_clone = app.clone();

    std::thread::spawn(move || match worker::run_pack(&project_clone) {
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

#[tauri::command]
pub fn pack(_state: State<'_, Mutex<TauriState>>, app: AppHandle) -> Result<(), String> {
    pack_impl(&app);
    Ok(())
}

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

#[tauri::command]
pub fn stop_watch(state: State<'_, Mutex<TauriState>>) -> Result<(), String> {
    let mut guard = state.lock().unwrap();
    guard.watcher = None;
    guard.log_info("Watch mode stopped.");
    Ok(())
}
