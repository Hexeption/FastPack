use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;

use crate::{
    compressor::{CompressInput, CompressOutput, Compressor},
    error::CompressError,
};

/// JPEG encoder. Quality is taken from `CompressInput::quality` (0–100).
///
/// When built with the `jpeg-turbo` cargo feature, mozjpeg is used for better
/// compression at the same quality setting. Without it, the `image` crate's
/// built-in encoder is used.
pub struct JpegCompressor;

impl Compressor for JpegCompressor {
    fn compress(&self, input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
        compress_jpeg(input)
    }

    fn format_id(&self) -> &'static str {
        "jpeg"
    }

    fn file_extension(&self) -> &'static str {
        "jpg"
    }
}

fn compress_jpeg(input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
    let quality = input.quality;

    #[cfg(feature = "jpeg-turbo")]
    return compress_mozjpeg(input.image, quality);

    let rgb = input.image.to_rgb8();
    let mut buf = Cursor::new(Vec::new());
    let mut encoder = JpegEncoder::new_with_quality(&mut buf, quality);
    encoder.encode(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(CompressOutput {
        data: buf.into_inner(),
    })
}

#[cfg(feature = "jpeg-turbo")]
fn compress_mozjpeg(
    image: &image::DynamicImage,
    quality: u8,
) -> Result<CompressOutput, CompressError> {
    let rgb = image.to_rgb8();
    let (width, height) = rgb.dimensions();
    let raw = rgb.as_raw();

    let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
    comp.set_size(width as usize, height as usize);
    comp.set_quality(quality as f32);
    comp.set_mem_dest();
    comp.start_compress();
    if !comp.write_scanlines(raw) {
        return Err(CompressError::Other(
            "mozjpeg write_scanlines failed: unexpected data length".to_string(),
        ));
    }
    let data = comp.finish_compress();
    Ok(CompressOutput { data })
}
