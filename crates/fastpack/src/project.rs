use std::{fs, path::Path};

use anyhow::{Context, Result};
use fastpack_core::types::config::Project;

/// Load a `.fpsheet` TOML project file.
pub fn load(path: &Path) -> Result<Project> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

/// Save a project to a `.fpsheet` TOML file.
pub fn save(project: &Project, path: &Path) -> Result<()> {
    let text = toml::to_string_pretty(project).context("failed to serialize project")?;
    fs::write(path, text.as_bytes()).with_context(|| format!("failed to write {}", path.display()))
}
