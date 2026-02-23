use fastpack_core::types::config::PackMode;

use crate::{
    compressor::{CompressInput, CompressOutput, Compressor},
    error::CompressError,
};

/// WebP encoder.
///
/// Requires the `webp-encode` cargo feature. `PackMode::Best` produces lossless
/// output; `Fast` and `Good` produce lossy output at the quality specified by
/// `CompressInput::quality`.
pub struct WebpCompressor;

impl Compressor for WebpCompressor {
    fn compress(&self, input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
        compress_webp(input)
    }

    fn format_id(&self) -> &'static str {
        "webp"
    }

    fn file_extension(&self) -> &'static str {
        "webp"
    }
}

fn compress_webp(input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
    #[cfg(feature = "webp-encode")]
    {
        let rgba = input.image.to_rgba8();
        let (width, height) = rgba.dimensions();
        let encoder = webp::Encoder::from_rgba(rgba.as_raw(), width, height);
        let output = match input.pack_mode {
            PackMode::Best => encoder.encode_lossless(),
            _ => encoder.encode(input.quality as f32),
        };
        return Ok(CompressOutput {
            data: output.to_vec(),
        });
    }

    #[cfg(not(feature = "webp-encode"))]
    Err(CompressError::Other(
        "webp-encode feature is not enabled; rebuild with --features webp-encode".to_string(),
    ))
}
