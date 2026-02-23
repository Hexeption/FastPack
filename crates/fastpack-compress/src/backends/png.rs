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

/// Lossy PNG encoder using imagequant palette reduction followed by oxipng compression.
///
/// `CompressInput::quality` (0–100) controls colour fidelity; lower values produce
/// smaller files. After palette reduction the result is passed through oxipng at
/// the same effort level as `PngCompressor`.
///
/// Requires the `png` cargo feature (enabled by default).
#[cfg(feature = "png")]
pub struct LossyPngCompressor;

#[cfg(feature = "png")]
impl Compressor for LossyPngCompressor {
    fn compress(&self, input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
        compress_lossy_png(input)
    }

    fn format_id(&self) -> &'static str {
        "png"
    }

    fn file_extension(&self) -> &'static str {
        "png"
    }
}

#[cfg(feature = "png")]
fn compress_lossy_png(input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
    let rgba = input.image.to_rgba8();
    let (w, h) = rgba.dimensions();
    let quality = input.quality;

    let pixels: Vec<imagequant::RGBA> = rgba
        .pixels()
        .map(|p| imagequant::RGBA {
            r: p[0],
            g: p[1],
            b: p[2],
            a: p[3],
        })
        .collect();

    let mut liq = imagequant::new();
    liq.set_quality(0, quality)
        .map_err(|e| CompressError::Other(e.to_string()))?;
    let mut img = liq
        .new_image_borrowed(&pixels, w as usize, h as usize, 0.0)
        .map_err(|e| CompressError::Other(e.to_string()))?;
    let mut res = liq
        .quantize(&mut img)
        .map_err(|e| CompressError::Other(e.to_string()))?;
    res.set_dithering_level(1.0)
        .map_err(|e| CompressError::Other(e.to_string()))?;
    let (palette, indexed) = res
        .remapped(&mut img)
        .map_err(|e| CompressError::Other(e.to_string()))?;

    let mut png_bytes = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut png_bytes, w, h);
        enc.set_color(png::ColorType::Indexed);
        enc.set_depth(png::BitDepth::Eight);
        let pal: Vec<u8> = palette.iter().flat_map(|c| [c.r, c.g, c.b]).collect();
        enc.set_palette(pal);
        let trns: Vec<u8> = palette.iter().map(|c| c.a).collect();
        enc.set_trns(trns);
        let mut writer = enc
            .write_header()
            .map_err(|e| CompressError::Other(e.to_string()))?;
        writer
            .write_image_data(&indexed)
            .map_err(|e| CompressError::Other(e.to_string()))?;
    }

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
