//! User preferences persisted to `~/.config/FastPack/prefs.toml`.
//!
//! Stores UI settings, keybinds, default packer config, and language choice.
//! Loaded once at startup and re-saved whenever the user changes them.

use std::path::PathBuf;

use fastpack_core::types::config::PackerConfig;
use serde::{Deserialize, Serialize};

/// A single keyboard shortcut binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybind {
    /// The key character (e.g. `"s"`, `"n"`).
    pub key: String,
    /// True if the platform modifier (Cmd on macOS, Ctrl elsewhere) is required.
    pub modifier: bool,
    /// True if Shift is required.
    pub shift: bool,
}

/// Keyboard shortcut assignments for common actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindsConfig {
    #[serde(default = "default_kb_new_project")]
    pub new_project: Keybind,
    #[serde(default = "default_kb_open_project")]
    pub open_project: Keybind,
    #[serde(default = "default_kb_save_project")]
    pub save_project: Keybind,
    #[serde(default = "default_kb_save_project_as")]
    pub save_project_as: Keybind,
    #[serde(default = "default_kb_anim_preview")]
    pub anim_preview: Keybind,
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            new_project: default_kb_new_project(),
            open_project: default_kb_open_project(),
            save_project: default_kb_save_project(),
            save_project_as: default_kb_save_project_as(),
            anim_preview: default_kb_anim_preview(),
        }
    }
}

fn default_kb_new_project() -> Keybind {
    Keybind {
        key: "n".into(),
        modifier: true,
        shift: false,
    }
}
fn default_kb_open_project() -> Keybind {
    Keybind {
        key: "o".into(),
        modifier: true,
        shift: false,
    }
}
fn default_kb_save_project() -> Keybind {
    Keybind {
        key: "s".into(),
        modifier: true,
        shift: false,
    }
}
fn default_kb_save_project_as() -> Keybind {
    Keybind {
        key: "s".into(),
        modifier: true,
        shift: true,
    }
}
fn default_kb_anim_preview() -> Keybind {
    Keybind {
        key: "p".into(),
        modifier: false,
        shift: false,
    }
}

/// Supported UI languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    En,
    Fr,
    Es,
    De,
    It,
    Pt,
    Ja,
    Zh,
    Ko,
}

impl Language {
    /// Return the IETF language tag (e.g. `"en"`, `"ja"`).
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
            Self::Es => "es",
            Self::De => "de",
            Self::It => "it",
            Self::Pt => "pt",
            Self::Ja => "ja",
            Self::Zh => "zh",
            Self::Ko => "ko",
        }
    }

    /// Return the native display name of the language.
    pub fn display(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Fr => "Français",
            Self::Es => "Español",
            Self::De => "Deutsch",
            Self::It => "Italiano",
            Self::Pt => "Português",
            Self::Ja => "日本語",
            Self::Zh => "中文（简体）",
            Self::Ko => "한국어",
        }
    }

    /// All supported language variants.
    pub const ALL: &'static [Language] = &[
        Self::En,
        Self::Fr,
        Self::Es,
        Self::De,
        Self::It,
        Self::Pt,
        Self::Ja,
        Self::Zh,
        Self::Ko,
    ];
}

/// Persistent user preferences saved to `~/.config/FastPack/prefs.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    /// Use dark theme when true.
    #[serde(default = "default_true")]
    pub dark_mode: bool,
    /// Check GitHub for new releases on startup.
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,
    /// Default packer config applied to new projects.
    #[serde(default)]
    pub default_config: PackerConfig,
    /// UI language.
    #[serde(default)]
    pub language: Language,
    /// Global UI scale factor (1.0 = 100%).
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    /// Keyboard shortcut assignments.
    #[serde(default)]
    pub keybinds: KeybindsConfig,
    /// Scroll speed multiplier for atlas zoom.
    #[serde(default = "default_zoom_speed")]
    pub atlas_zoom_speed: f32,
    /// Invert scroll direction when zooming the atlas.
    #[serde(default)]
    pub atlas_invert_scroll: bool,
}

fn default_true() -> bool {
    true
}

fn default_ui_scale() -> f32 {
    1.0
}

fn default_zoom_speed() -> f32 {
    1.0
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            dark_mode: true,
            auto_check_updates: true,
            default_config: PackerConfig::default(),
            language: Language::En,
            ui_scale: 1.0,
            keybinds: KeybindsConfig::default(),
            atlas_zoom_speed: 1.0,
            atlas_invert_scroll: false,
        }
    }
}

impl Preferences {
    /// Load preferences from disk, returning defaults if the file is missing.
    pub fn load() -> Self {
        prefs_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Persist the current preferences to disk.
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

/// Resolve the preferences file path. Returns `None` if the config dir is unknown.
fn prefs_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("FastPack").join("prefs.toml"))
}
