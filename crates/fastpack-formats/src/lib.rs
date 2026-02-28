//! Atlas metadata export formats (JSON Hash, JSON Array, Phaser 3, PixiJS).
//!
//! Each format implements the [`exporter::Exporter`] trait: serialize a
//! [`PackedAtlas`](fastpack_core::types::atlas::PackedAtlas) into a data file
//! string. The CLI and GUI pick a format by name and call `export()`.

#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

pub mod error;
pub mod exporter;
pub mod formats;
