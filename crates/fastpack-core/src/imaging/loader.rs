use std::hash::Hasher;
use std::path::{Path, PathBuf};

use image::DynamicImage;
use rayon::prelude::*;
use rustc_hash::FxHasher;

use crate::{
    error::CoreError,
    types::{rect::Size, sprite::Sprite},
};

/// Load a single image file and return a [`Sprite`] with normalized RGBA8 pixel data.
///
/// Supports PNG, JPEG, BMP, TGA, WebP, and TIFF natively via the `image` crate.
/// With the `svg` feature enabled, SVG files are rasterized at their natural viewport size.
/// With the `psd` feature enabled, PSDs are flattened to a single RGBA layer.
///
/// The returned `Sprite` has `trim_rect`, `polygon`, `nine_patch`, `pivot`, and
/// `alias_of` all set to `None`. Those fields are filled by later pipeline stages.
pub fn load(path: &Path, id: impl Into<String>) -> Result<Sprite, CoreError> {
    let id = id.into();
    let image = decode(path)?;
    let rgba = image.into_rgba8();
    let (w, h) = rgba.dimensions();
    let content_hash = hash_rgba(&rgba);
    Ok(Sprite {
        id,
        source_path: path.to_path_buf(),
        image: DynamicImage::ImageRgba8(rgba),
        trim_rect: None,
        original_size: Size { w, h },
        polygon: None,
        nine_patch: None,
        pivot: None,
        content_hash,
        alias_of: None,
    })
}

/// Load multiple sprites in parallel using rayon.
///
/// Each entry is `(absolute_path, sprite_id)`. Results are returned in input order.
/// Failed loads are `Err` entries; callers decide whether to abort or skip them.
pub fn load_many(paths: &[(PathBuf, String)]) -> Vec<Result<Sprite, CoreError>> {
    paths
        .par_iter()
        .map(|(path, id)| load(path, id.clone()))
        .collect()
}

fn decode(path: &Path) -> Result<DynamicImage, CoreError> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        #[cfg(feature = "svg")]
        "svg" => decode_svg(path),
        #[cfg(feature = "psd")]
        "psd" => decode_psd(path),
        _ => image::open(path).map_err(|source| CoreError::ImageLoad {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn hash_rgba(img: &image::RgbaImage) -> u64 {
    let mut h = FxHasher::default();
    h.write(img.as_raw());
    h.finish()
}

#[cfg(feature = "svg")]
fn decode_svg(path: &Path) -> Result<DynamicImage, CoreError> {
    let data = std::fs::read(path).map_err(CoreError::Io)?;
    let opts = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&data, &opts).map_err(|e| {
        CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))
    })?;
    let w = tree.size().width().ceil() as u32;
    let h = tree.size().height().ceil() as u32;
    if w == 0 || h == 0 {
        return Err(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "SVG has zero-size viewport",
        )));
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or_else(|| {
        CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "could not allocate SVG pixmap",
        ))
    })?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    // tiny_skia produces premultiplied RGBA; convert to straight alpha for consistency.
    let mut raw = pixmap.take();
    demultiply_alpha(&mut raw);
    image::RgbaImage::from_raw(w, h, raw)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| {
            CoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "SVG pixmap buffer size mismatch",
            ))
        })
}

#[cfg(feature = "svg")]
fn demultiply_alpha(data: &mut [u8]) {
    for pixel in data.chunks_exact_mut(4) {
        let a = pixel[3];
        if a == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        } else if a < 255 {
            let inv = 255.0 / a as f32;
            pixel[0] = (pixel[0] as f32 * inv).round().min(255.0) as u8;
            pixel[1] = (pixel[1] as f32 * inv).round().min(255.0) as u8;
            pixel[2] = (pixel[2] as f32 * inv).round().min(255.0) as u8;
        }
    }
}

#[cfg(feature = "psd")]
fn decode_psd(path: &Path) -> Result<DynamicImage, CoreError> {
    let data = std::fs::read(path).map_err(CoreError::Io)?;
    let psd_file = psd::Psd::from_bytes(&data).map_err(|e| {
        CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))
    })?;
    let w = psd_file.width();
    let h = psd_file.height();
    image::RgbaImage::from_raw(w, h, psd_file.rgba())
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| {
            CoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "PSD buffer size mismatch",
            ))
        })
}
