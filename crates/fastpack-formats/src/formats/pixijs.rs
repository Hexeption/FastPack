use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter},
    formats::json_hash::JsonHashExporter,
};

/// Exports atlas metadata in PixiJS compatible format.
///
/// PixiJS uses the JSON Hash layout. This exporter delegates directly to
/// [`JsonHashExporter`].
pub struct PixiJsExporter;

impl Exporter for PixiJsExporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        JsonHashExporter.export(input)
    }

    fn format_id(&self) -> &'static str {
        "pixijs"
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}
