use std::io::Cursor;

use fastpack_core::types::config::PackMode;
use image::ImageFormat;

use crate::{
    compressor::{CompressInput, CompressOutput, Compressor},
    error::CompressError,
};

/// Lossless PNG encoder backed by the `image` crate with optional oxipng optimization.
///
/// `PackMode::Fast` encodes only (no post-processing).
/// `PackMode::Good` runs oxipng at preset level 3.
/// `PackMode::Best` runs oxipng at preset level 6 (maximum libdeflater compression).
pub struct PngCompressor;

impl Compressor for PngCompressor {
    fn compress(&self, input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
        compress_png(input)
    }

    fn format_id(&self) -> &'static str {
        "png"
    }

    fn file_extension(&self) -> &'static str {
        "png"
    }
}

fn compress_png(input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
    let mut buf = Cursor::new(Vec::new());
    input.image.write_to(&mut buf, ImageFormat::Png)?;
    let png_bytes = buf.into_inner();

    #[cfg(feature = "png")]
    match input.pack_mode {
        PackMode::Fast => {}
        PackMode::Good => {
            let opts = oxipng::Options::from_preset(3);
            let optimized = oxipng::optimize_from_memory(&png_bytes, &opts)?;
            return Ok(CompressOutput { data: optimized });
        }
        PackMode::Best => {
            let opts = oxipng::Options::from_preset(6);
            let optimized = oxipng::optimize_from_memory(&png_bytes, &opts)?;
            return Ok(CompressOutput { data: optimized });
        }
    }

    Ok(CompressOutput { data: png_bytes })
}
