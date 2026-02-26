use std::path::PathBuf;

use fastpack_core::types::{
    atlas::AtlasFrame,
    config::{PackerConfig, Project, SourceSpec},
};
use rust_i18n::t;

/// Severity level for a log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Informational message.
    Info,
    /// Non-fatal warning.
    Warn,
    /// Operation failure.
    Error,
}

/// A single timestamped output log entry.
pub struct LogEntry {
    /// Severity of the message.
    pub level: LogLevel,
    /// Human-readable message text.
    pub message: String,
    /// Local time string in HH:MM:SS format.
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
    /// Create an info-level log entry.
    pub fn info(msg: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Info,
            message: msg.into(),
            time: format_time(),
        }
    }
    /// Create a warn-level log entry.
    pub fn warn(msg: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Warn,
            message: msg.into(),
            time: format_time(),
        }
    }
    /// Create an error-level log entry.
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Error,
            message: msg.into(),
            time: format_time(),
        }
    }
}

/// A single frame in the packed atlas (used by sprite list + preview).
#[derive(Clone)]
pub struct FrameInfo {
    /// Sprite identifier used in export data.
    pub id: String,
    /// Index of the atlas sheet this frame lives on.
    pub sheet_idx: usize,
    /// Packed X position in atlas pixels.
    pub x: u32,
    /// Packed Y position in atlas pixels.
    pub y: u32,
    /// Packed frame width in pixels.
    pub w: u32,
    /// Packed frame height in pixels.
    pub h: u32,
    /// Set to the canonical sprite ID if this frame is a duplicate.
    pub alias_of: Option<String>,
}

/// Per-sheet atlas data kept in AppState after a pack run.
pub struct SheetData {
    /// Raw RGBA pixel data.
    pub rgba: Vec<u8>,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// UI-facing frame metadata for this sheet.
    pub frames: Vec<FrameInfo>,
    /// Full atlas frame data used by exporters.
    pub atlas_frames: Vec<AtlasFrame>,
}

/// One-shot flags set by menus/toolbar and consumed by the app's update loop.
#[derive(Default)]
pub struct PendingActions {
    /// Trigger a new pack run.
    pub pack: bool,
    /// Export the last packed result.
    pub export: bool,
    /// Clear state and start a new project.
    pub new_project: bool,
    /// Open an existing `.fpsheet` file.
    pub open_project: bool,
    /// Save the project to its current path.
    pub save_project: bool,
    /// Save the project to a user-chosen path.
    pub save_project_as: bool,
    /// Open a folder picker and add a source directory.
    pub add_source: bool,
    /// Open the preferences window.
    pub open_prefs: bool,
    /// Rebuild the filesystem watcher to match current sources.
    pub rebuild_watcher: bool,
}

/// Playback state for the animation preview window.
pub struct AnimPreviewState {
    /// Whether the preview window is open.
    pub open: bool,
    /// Whether playback is running.
    pub playing: bool,
    /// Frames per second for playback.
    pub fps: f32,
    /// Whether to loop back to the first frame after the last.
    pub looping: bool,
    /// Index into `AppState.selected_frames` for the currently displayed frame.
    pub current_frame: usize,
    /// Accumulated time since the last frame advance (seconds).
    pub elapsed_secs: f64,
    /// Zoom scale for the canvas (1.0 = pixel-perfect).
    pub zoom: f32,
    /// Pan offset for the canvas (screen pixels from centre).
    pub pan: [f32; 2],
}

impl Default for AnimPreviewState {
    fn default() -> Self {
        Self {
            open: false,
            playing: false,
            fps: 24.0,
            looping: true,
            current_frame: 0,
            elapsed_secs: 0.0,
            zoom: 1.0,
            pan: [0.0, 0.0],
        }
    }
}

/// All runtime state shared across the GUI.
pub struct AppState {
    /// Project configuration and source specs (serialised to/from .fpsheet).
    pub project: Project,
    /// Path of the currently open .fpsheet file, if any.
    pub project_path: Option<PathBuf>,
    /// True when there are unsaved changes.
    pub dirty: bool,

    /// Messages shown in the output log panel.
    pub log: Vec<LogEntry>,

    /// All packed sheets from the last successful pack.
    pub sheets: Vec<SheetData>,
    /// Frame entries from all sheets concatenated, in sheet order.
    pub frames: Vec<FrameInfo>,
    /// Counts from the last pack run.
    pub sprite_count: usize,
    /// Sprites deduplicated as aliases in the last pack.
    pub alias_count: usize,
    /// Sprites that did not fit when multipack is disabled.
    pub overflow_count: usize,

    /// True while a pack is running in the background.
    pub packing: bool,

    /// Indices into `self.frames` of highlighted frames, in click order.
    pub selected_frames: Vec<usize>,
    /// Anchor frame for shift+click range selection. Set on plain click.
    pub anchor_frame: Option<usize>,
    /// State for the animation preview window.
    pub anim_preview: AnimPreviewState,

    /// Pan offset for the atlas preview (screen pixels from the centre).
    pub atlas_pan: [f32; 2],
    /// Zoom scale for the atlas preview (1.0 = pixel-perfect).
    pub atlas_zoom: f32,

    /// True for the custom dark theme, false for light.
    pub dark_mode: bool,

    /// One-shot actions queued by menus and toolbar, processed at frame start.
    pub pending: PendingActions,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            project: Project::default(),
            project_path: None,
            dirty: false,
            log: Vec::new(),
            sheets: Vec::new(),
            frames: Vec::new(),
            sprite_count: 0,
            alias_count: 0,
            overflow_count: 0,
            packing: false,
            selected_frames: Vec::new(),
            anchor_frame: None,
            anim_preview: AnimPreviewState::default(),
            atlas_pan: [0.0, 0.0],
            atlas_zoom: 1.0,
            dark_mode: true,
            pending: PendingActions::default(),
        }
    }
}

impl AppState {
    /// Append an info message to the output log.
    pub fn log_info(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::info(msg));
    }
    /// Append a warning message to the output log.
    pub fn log_warn(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::warn(msg));
    }
    /// Append an error message to the output log.
    pub fn log_error(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::error(msg));
    }

    /// Window title with project name and dirty marker.
    pub fn window_title(&self) -> String {
        let name = self
            .project_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| t!("state.untitled").to_string());
        if self.dirty {
            format!("FastPack — {}*", name)
        } else {
            format!("FastPack — {}", name)
        }
    }

    /// Discard all state and start fresh using the given default config.
    pub fn new_project(&mut self, default_config: PackerConfig) {
        let dark_mode = self.dark_mode;
        *self = AppState::default();
        self.dark_mode = dark_mode;
        self.project.config = default_config;
        self.log.push(LogEntry::info(t!("state.new_project")));
    }

    /// Add a source directory and schedule an auto-pack. No-ops if already tracked.
    pub fn add_source_path(&mut self, path: PathBuf) {
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        // Skip if any existing source already covers this path (same dir or parent of it).
        if self.project.sources.iter().any(|s| {
            let stored = std::fs::canonicalize(&s.path).unwrap_or_else(|_| s.path.clone());
            path.starts_with(&stored)
        }) {
            return;
        }
        let display = path.display().to_string();
        self.project.sources.push(SourceSpec {
            path,
            filter: "**/*.png".to_string(),
        });
        self.dirty = true;
        self.pending.pack = true;
        self.pending.rebuild_watcher = true;
        self.log_info(t!("state.added_source", path = display));
    }

    /// Remove the source at `index`.
    pub fn remove_source(&mut self, index: usize) {
        if index < self.project.sources.len() {
            let removed = self.project.sources.remove(index);
            self.dirty = true;
            self.pending.pack = true;
            self.pending.rebuild_watcher = true;
            self.log_info(t!(
                "state.removed_source",
                path = removed.path.display().to_string()
            ));
        }
    }
}
