use std::path::PathBuf;

use fastpack_core::types::config::PackerConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    En,
    /// French.
    Fr,
    /// Spanish.
    Es,
    /// German.
    De,
    /// Italian.
    It,
    /// Portuguese.
    Pt,
    /// Japanese.
    Ja,
    /// Simplified Chinese.
    Zh,
    /// Korean.
    Ko,
}

impl Language {
    /// Return the BCP-47 locale code for this language.
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

    /// Return the native display name for this language.
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

    /// All supported languages in menu order.
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
    /// Enable the dark UI theme.
    #[serde(default = "default_true")]
    pub dark_mode: bool,
    /// Check for a newer release at startup.
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,
    /// Default project configuration applied to new projects.
    #[serde(default)]
    pub default_config: PackerConfig,
    /// UI display language.
    #[serde(default)]
    pub language: Language,
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
            language: Language::En,
        }
    }
}

impl Preferences {
    /// Load preferences from disk, returning defaults if the file is missing or unreadable.
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

fn prefs_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("FastPack").join("prefs.toml"))
}
