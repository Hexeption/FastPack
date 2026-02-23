use anyhow::{Result, bail};
use image::{GenericImageView, imageops::FilterType};

use crate::types::{
    config::ScaleMode,
    rect::{Point, Size, SourceRect},
    sprite::{NinePatch, Sprite},
};

/// Produce a copy of `sprite` scaled by `factor` using the given resampling mode.
///
/// `factor < 1.0` shrinks; `factor > 1.0` enlarges. When `factor` is exactly
/// 1.0 the sprite is returned as-is (cloned, no resampling).
///
/// Pixel art modes (`Scale2x`, `Scale3x`, `Hq2x`, `Eagle`) are not yet
/// implemented and return an error; use `Smooth` or `Fast` for now.
pub fn scale_sprite(sprite: &Sprite, factor: f32, mode: ScaleMode) -> Result<Sprite> {
    if (factor - 1.0).abs() < f32::EPSILON {
        return Ok(sprite.clone());
    }

    match mode {
        ScaleMode::Scale2x | ScaleMode::Scale3x | ScaleMode::Hq2x | ScaleMode::Eagle => {
            bail!(
                "pixel art scale modes (scale2x/scale3x/hq2x/eagle) are not yet implemented; \
                 use smooth or fast"
            );
        }
        ScaleMode::Smooth | ScaleMode::Fast => {}
    }

    let filter = match mode {
        ScaleMode::Smooth => FilterType::Lanczos3,
        ScaleMode::Fast => FilterType::Nearest,
        _ => unreachable!(),
    };

    let (w, h) = sprite.image.dimensions();
    let new_w = ((w as f32 * factor).round() as u32).max(1);
    let new_h = ((h as f32 * factor).round() as u32).max(1);
    let scaled_image = sprite.image.resize_exact(new_w, new_h, filter);

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
