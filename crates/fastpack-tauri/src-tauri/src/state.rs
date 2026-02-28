//! Shared application state for the Tauri backend.
//!
//! Holds the current project, pack results, log history, and watcher handle.
//! Every Tauri command accesses this through a `Mutex<TauriState>`.

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

/// Format the current UTC time as `HH:MM:SS`.
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
    /// Create an info-level log entry with the current timestamp.
    pub fn info(msg: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Info,
            message: msg.into(),
            time: format_time(),
        }
    }
    /// Create a warn-level log entry with the current timestamp.
    pub fn warn(msg: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Warn,
            message: msg.into(),
            time: format_time(),
        }
    }
    /// Create an error-level log entry with the current timestamp.
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
    /// Sprite identifier (forward-slash path without extension).
    pub id: String,
    /// Absolute path of the source image file.
    pub src_path: String,
    /// X offset in the atlas texture.
    pub x: u32,
    /// Y offset in the atlas texture.
    pub y: u32,
    /// Frame width in pixels.
    pub w: u32,
    /// Frame height in pixels.
    pub h: u32,
    /// If this frame is an alias, the ID of the canonical sprite.
    pub alias_of: Option<String>,
}

/// Handle for a running filesystem watcher.
pub struct WatcherHandle {
    /// The debounced filesystem watcher instance.
    pub _debouncer:
        notify_debouncer_mini::Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>,
    /// Channel sender to stop the watcher. Dropping the handle also stops it.
    pub stop_tx: mpsc::SyncSender<()>,
}

/// All runtime state shared across Tauri commands.
pub struct TauriState {
    /// The current project configuration and sources.
    pub project: Project,
    /// Path to the `.fpsheet` file on disk. `None` for unsaved projects.
    pub project_path: Option<PathBuf>,
    /// True if the project has unsaved changes.
    pub dirty: bool,
    /// Ordered log entries shown in the UI console.
    pub log: Vec<LogEntry>,
    /// Packed sheet data from the last successful pack.
    pub sheets: Vec<SheetData>,
    /// Total number of sprites in the last pack.
    pub sprite_count: usize,
    /// Number of duplicate sprites detected as aliases.
    pub alias_count: usize,
    /// Number of sprites that did not fit in the atlas.
    pub overflow_count: usize,
    /// True while a pack or publish operation runs on a background thread.
    pub is_packing: bool,
    /// User preferences loaded from disk.
    pub prefs: Preferences,
    /// Active filesystem watcher for auto-repack. `None` when watch mode is off.
    pub watcher: Option<WatcherHandle>,
}

impl TauriState {
    /// Build initial state. If `project_path` is given, load that `.fpsheet` file.
    pub fn new(project_path: Option<PathBuf>) -> Self {
        let prefs = Preferences::load();
        let mut state = Self {
            project: Project {
                config: prefs.default_config.clone(),
                sources: Vec::new(),
                folder_colors: Default::default(),
            },
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

    /// Push an info-level message to the log.
    pub fn log_info(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::info(msg));
    }
    /// Push a warn-level message to the log.
    pub fn log_warn(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::warn(msg));
    }
    /// Push an error-level message to the log.
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
                src_path: f.src_path.clone(),
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
