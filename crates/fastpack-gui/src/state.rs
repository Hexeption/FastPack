use std::path::PathBuf;

use fastpack_core::types::config::PackerConfig;

/// Central application state shared across all panels.
pub struct AppState {
    pub config: PackerConfig,
    pub project_path: Option<PathBuf>,
    pub dirty: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: PackerConfig::default(),
            project_path: None,
            dirty: false,
        }
    }
}
