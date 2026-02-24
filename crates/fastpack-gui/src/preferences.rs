use std::path::PathBuf;

use fastpack_core::types::config::PackerConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(default = "default_true")]
    pub dark_mode: bool,
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,
    #[serde(default)]
    pub default_config: PackerConfig,
}

fn default_true() -> bool {
    true
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            dark_mode: true,
            auto_check_updates: true,
            default_config: PackerConfig::default(),
        }
    }
}

impl Preferences {
    pub fn load() -> Self {
        prefs_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = prefs_path() else { return };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, text.as_bytes());
        }
    }
}

fn prefs_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("FastPack").join("prefs.toml"))
}
