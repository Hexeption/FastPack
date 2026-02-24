use std::path::PathBuf;

use fastpack_core::types::{
    atlas::AtlasFrame,
    config::{Project, SourceSpec},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

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

/// A single frame in the packed atlas (used by sprite list + preview).
pub struct FrameInfo {
    pub id: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub alias_of: Option<String>,
}

/// One-shot flags set by menus/toolbar and consumed by the app's update loop.
#[derive(Default)]
pub struct PendingActions {
    pub pack: bool,
    pub export: bool,
    pub new_project: bool,
    pub open_project: bool,
    pub save_project: bool,
    pub save_project_as: bool,
    pub add_source: bool,
}

pub struct AppState {
    /// Project configuration and source specs (serialised to/from .fpsheet).
    pub project: Project,
    /// Path of the currently open .fpsheet file, if any.
    pub project_path: Option<PathBuf>,
    /// True when there are unsaved changes.
    pub dirty: bool,

    /// Messages shown in the output log panel.
    pub log: Vec<LogEntry>,

    /// Frame entries populated after each successful pack.
    pub frames: Vec<FrameInfo>,
    /// Raw RGBA8888 pixel data from the last pack, plus (width, height).
    pub atlas_rgba: Option<(Vec<u8>, u32, u32)>,
    /// Full atlas frame metadata for export.
    pub atlas_frames: Vec<AtlasFrame>,
    /// Counts from the last pack run.
    pub sprite_count: usize,
    pub alias_count: usize,
    pub overflow_count: usize,

    /// True while a pack is running in the background.
    pub packing: bool,

    /// Index into `self.frames` of the highlighted frame, if any.
    pub selected_frame: Option<usize>,

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
            frames: Vec::new(),
            atlas_rgba: None,
            atlas_frames: Vec::new(),
            sprite_count: 0,
            alias_count: 0,
            overflow_count: 0,
            packing: false,
            selected_frame: None,
            atlas_pan: [0.0, 0.0],
            atlas_zoom: 1.0,
            dark_mode: true,
            pending: PendingActions::default(),
        }
    }
}

impl AppState {
    pub fn log_info(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::info(msg));
    }
    pub fn log_warn(&mut self, msg: impl Into<String>) {
        self.log.push(LogEntry::warn(msg));
    }
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
            .unwrap_or_else(|| "Untitled".into());
        if self.dirty {
            format!("FastPack — {}*", name)
        } else {
            format!("FastPack — {}", name)
        }
    }

    /// Discard all state and start fresh.
    pub fn new_project(&mut self) {
        let dark_mode = self.dark_mode;
        *self = AppState::default();
        self.dark_mode = dark_mode;
        self.log.push(LogEntry::info("New project created."));
    }

    /// Add a source directory and schedule an auto-pack.
    pub fn add_source_path(&mut self, path: PathBuf) {
        let display = path.display().to_string();
        self.project.sources.push(SourceSpec {
            path,
            filter: "**/*.png".to_string(),
        });
        self.dirty = true;
        self.pending.pack = true;
        self.log_info(format!("Added source: {display}"));
    }

    /// Remove the source at `index`.
    pub fn remove_source(&mut self, index: usize) {
        if index < self.project.sources.len() {
            let removed = self.project.sources.remove(index);
            self.dirty = true;
            self.log_info(format!("Removed source: {}", removed.path.display()));
        }
    }
}
