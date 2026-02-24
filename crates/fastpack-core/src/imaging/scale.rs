use anyhow::Result;
use image::{DynamicImage, GenericImageView, imageops::FilterType};

use crate::types::{
    config::ScaleMode,
    rect::{Point, Size, SourceRect},
    sprite::{NinePatch, Sprite},
};

use super::pixel_art;

/// Produce a copy of `sprite` scaled by `factor` using the given resampling mode.
///
/// `factor < 1.0` shrinks; `factor > 1.0` enlarges. When `factor` is exactly
/// 1.0 the sprite is returned as-is (cloned, no resampling).
///
/// Pixel art modes (`Scale2x`, `Scale3x`, `Hq2x`, `Eagle`) first apply their
/// integer upscaler (2× or 3×) and then resize to the exact target dimensions
/// with nearest-neighbour when the factor does not align exactly with the
/// algorithm's native multiplier.
pub fn scale_sprite(sprite: &Sprite, factor: f32, mode: ScaleMode) -> Result<Sprite> {
    if (factor - 1.0).abs() < f32::EPSILON {
        return Ok(sprite.clone());
    }

    let (src_w, src_h) = sprite.image.dimensions();
    let target_w = ((src_w as f32 * factor).round() as u32).max(1);
    let target_h = ((src_h as f32 * factor).round() as u32).max(1);

    let scaled_image = match mode {
        ScaleMode::Smooth => sprite
            .image
            .resize_exact(target_w, target_h, FilterType::Lanczos3),
        ScaleMode::Fast => sprite
            .image
            .resize_exact(target_w, target_h, FilterType::Nearest),
        ScaleMode::Scale2x => {
            let up = pixel_art::scale2x(&sprite.image.to_rgba8());
            resize_to(DynamicImage::ImageRgba8(up), target_w, target_h)
        }
        ScaleMode::Scale3x => {
            let up = pixel_art::scale3x(&sprite.image.to_rgba8());
            resize_to(DynamicImage::ImageRgba8(up), target_w, target_h)
        }
        ScaleMode::Hq2x => {
            let up = pixel_art::hq2x(&sprite.image.to_rgba8());
            resize_to(DynamicImage::ImageRgba8(up), target_w, target_h)
        }
        ScaleMode::Eagle => {
            let up = pixel_art::eagle2x(&sprite.image.to_rgba8());
            resize_to(DynamicImage::ImageRgba8(up), target_w, target_h)
        }
    };

    let original_size = Size {
        w: ((sprite.original_size.w as f32 * factor).round() as u32).max(1),
        h: ((sprite.original_size.h as f32 * factor).round() as u32).max(1),
    };

    let trim_rect = sprite.trim_rect.as_ref().map(|tr| SourceRect {
        x: (tr.x as f32 * factor).round() as i32,
        y: (tr.y as f32 * factor).round() as i32,
        w: ((tr.w as f32 * factor).round() as u32).max(1),
        h: ((tr.h as f32 * factor).round() as u32).max(1),
    });

    let nine_patch = sprite.nine_patch.map(|np| NinePatch {
        top: (np.top as f32 * factor).round() as u32,
        right: (np.right as f32 * factor).round() as u32,
        bottom: (np.bottom as f32 * factor).round() as u32,
        left: (np.left as f32 * factor).round() as u32,
    });

    let polygon = sprite.polygon.as_ref().map(|pts| {
        pts.iter()
            .map(|p| Point {
                x: p.x * factor,
                y: p.y * factor,
            })
            .collect()
    });

    Ok(Sprite {
        id: sprite.id.clone(),
        source_path: sprite.source_path.clone(),
        image: scaled_image,
        trim_rect,
        original_size,
        polygon,
        nine_patch,
        pivot: sprite.pivot,
        content_hash: sprite.content_hash,
        extrude: (sprite.extrude as f32 * factor).round() as u32,
        alias_of: sprite.alias_of.clone(),
    })
}

/// Resize `img` to `(w, h)` using nearest-neighbour, or return it unchanged
/// when the dimensions already match.
fn resize_to(img: DynamicImage, w: u32, h: u32) -> DynamicImage {
    if img.width() == w && img.height() == h {
        img
    } else {
        img.resize_exact(w, h, FilterType::Nearest)
    }
}
