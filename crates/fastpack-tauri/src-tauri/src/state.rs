use std::path::PathBuf;
use std::sync::mpsc;

use fastpack_core::types::config::Project;
use serde::{Deserialize, Serialize};

use crate::preferences::Preferences;
use crate::worker::SheetOutput;

/// Severity level for a log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

/// A timestamped log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub time: String,
}

fn format_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

impl LogEntry {
    pub fn info(msg: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Info,
            message: msg.into(),
            time: format_time(),
        }
    }
    pub fn warn(msg: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Warn,
            message: msg.into(),
            time: format_time(),
        }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Error,
            message: msg.into(),
            time: format_time(),
        }
    }
}

/// Packed sheet data for the frontend (RGBA encoded as base64 PNG).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetData {
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// PNG image encoded as base64.
    pub png_b64: String,
    /// Frame metadata for overlay rendering.
    pub frames: Vec<FrameData>,
}

/// Packed frame metadata sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    pub id: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub alias_of: Option<String>,
}

/// Handle for a running filesystem watcher.
pub struct WatcherHandle {
    pub _debouncer:
        notify_debouncer_mini::Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>,
    pub stop_tx: mpsc::SyncSender<()>,
}

/// All runtime state shared across Tauri commands.
pub struct TauriState {
    pub project: Project,
    pub project_path: Option<PathBuf>,
    pub dirty: bool,
    pub log: Vec<LogEntry>,
    pub sheets: Vec<SheetData>,
    pub sprite_count: usize,
    pub alias_count: usize,
    pub overflow_count: usize,
    pub is_packing: bool,
    pub prefs: Preferences,
    pub watcher: Option<WatcherHandle>,
}

impl TauriState {
    pub fn new(project_path: Option<PathBuf>) -> Self {
        let prefs = Preferences::load();
        let mut state = Self {
            project: Project::default(),
            project_path: None,
            dirty: false,
            log: Vec::new(),
            sheets: Vec::new(),
            sprite_count: 0,
            alias_count: 0,
            overflow_count: 0,
            is_packing: false,
            prefs,
            watcher: None,
        };

        if let Some(path) = project_path {
            match std::fs::read_to_string(&path) {
                Ok(text) => match toml::from_str(&text) {
                    Ok(project) => {
                        state.project = project;
                        state.project_path = Some(path);
                        state.log.push(LogEntry::info("Project loaded."));
                    }
                    Err(e) => state.log.push(LogEntry::error(format!("Parse error: {e}"))),
                },
                Err(e) => state.log.push(LogEntry::error(format!("Read error: {e}"))),
            }
        }

        state
    }

    pub fn log_info(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::info(msg));
    }
    pub fn log_warn(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::warn(msg));
    }
    pub fn log_error(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::error(msg));
    }

    /// Convert a raw SheetOutput into SheetData by PNG-encoding the RGBA buffer.
    pub fn sheet_to_data(sheet: &SheetOutput) -> SheetData {
        use base64::Engine;
        use image::{ImageBuffer, Rgba};

        let img: ImageBuffer<Rgba<u8>, _> =
            ImageBuffer::from_raw(sheet.width, sheet.height, sheet.rgba.clone())
                .expect("valid rgba buffer");

        let mut png_bytes: Vec<u8> = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .expect("png encode");

        let png_b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

        let frames = sheet
            .frames
            .iter()
            .map(|f| FrameData {
                id: f.id.clone(),
                x: f.x,
                y: f.y,
                w: f.w,
                h: f.h,
                alias_of: f.alias_of.clone(),
            })
            .collect();

        SheetData {
            width: sheet.width,
            height: sheet.height,
            png_b64,
            frames,
        }
    }
}
