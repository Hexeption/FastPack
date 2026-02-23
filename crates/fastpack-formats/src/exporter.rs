use fastpack_core::types::atlas::PackedAtlas;

use crate::error::FormatError;

/// Input passed to every export format writer.
pub struct ExportInput<'a> {
    /// The packed atlas containing frames and metadata.
    pub atlas: &'a PackedAtlas,

    /// Texture filename written into the data file's image reference (e.g. `"atlas.png"`).
    pub texture_filename: String,

    /// Pixel format string written into the data file (e.g. `"RGBA8888"`).
    pub pixel_format: String,
}

/// Common interface for all export format writers.
pub trait Exporter: Send + Sync {
    /// Serialize atlas metadata and return the full data file content as a string.
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError>;

    /// Short identifier used in `.fpsheet` `data_format` field (e.g. `"json_hash"`).
    fn format_id(&self) -> &'static str;

    /// File extension for the output data file, without leading dot (e.g. `"json"`).
    fn file_extension(&self) -> &'static str;

    /// Combine multiple sheets into one data file where the format supports it.
    ///
    /// Returns `Some(content)` when this exporter can write all sheets as a single
    /// file (e.g. Phaser 3 multi-atlas `textures` array). Returns `None` to fall back
    /// to calling `export()` once per sheet.
    fn combine(&self, inputs: &[ExportInput<'_>]) -> Option<Result<String, FormatError>> {
        let _ = inputs;
        None
    }
}
