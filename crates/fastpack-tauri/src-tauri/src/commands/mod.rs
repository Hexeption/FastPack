//! Tauri command handlers invoked from the frontend via `invoke()`.
//!
//! Each submodule groups related commands: project management, packing,
//! dialogs, preferences, updater, and CLI installation.

pub mod cli;
pub mod dialogs;
pub mod pack;
pub mod preferences;
pub mod project;
pub mod updater;
