use fastpack_core::types::config::PackMode;
use image::DynamicImage;

use crate::error::CompressError;

/// Input to a compression backend.
pub struct CompressInput<'a> {
    /// The composited atlas image to encode.
    pub image: &'a DynamicImage,

    /// Controls the compression effort level.
    pub pack_mode: PackMode,
}

/// Raw encoded bytes returned by a compression backend.
pub struct CompressOutput {
    pub data: Vec<u8>,
}

/// Common interface for all image compression backends.
pub trait Compressor: Send + Sync {
    /// Encode the atlas image and return the file bytes.
    fn compress(&self, input: &CompressInput<'_>) -> Result<CompressOutput, CompressError>;

    /// Short identifier matching `OutputConfig::texture_format` (e.g. `"png"`).
    fn format_id(&self) -> &'static str;

    /// File extension for the texture output, without leading dot (e.g. `"png"`).
    fn file_extension(&self) -> &'static str;
}
