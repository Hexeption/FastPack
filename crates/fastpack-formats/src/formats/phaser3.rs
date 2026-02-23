use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter},
    formats::json_array::JsonArrayExporter,
};

/// Exports atlas metadata in Phaser 3 compatible format.
///
/// Phaser 3 uses the JSON Array layout. This exporter delegates directly to
/// [`JsonArrayExporter`].
pub struct Phaser3Exporter;

impl Exporter for Phaser3Exporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        JsonArrayExporter.export(input)
    }

    fn format_id(&self) -> &'static str {
        "phaser3"
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}
