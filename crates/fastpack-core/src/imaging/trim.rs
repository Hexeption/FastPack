use std::hash::Hasher;

use image::DynamicImage;
use rustc_hash::FxHasher;

use crate::types::{
    config::{SpriteConfig, TrimMode},
    rect::SourceRect,
    sprite::Sprite,
};

/// Trim transparent borders from a sprite's image.
///
/// Modifies `sprite.image`, `sprite.trim_rect`, and — for `TrimMode::Polygon` —
/// `sprite.polygon` in place. `sprite.original_size` is left unchanged.
///
/// For `TrimMode::None` the function is a no-op. For all other modes the image
/// is cropped to the opaque bounding box (plus `trim_margin`). Polygon hull
/// computation is deferred to Phase 3; until then `Polygon` behaves like `Trim`.
pub fn trim(sprite: &mut Sprite, config: &SpriteConfig) {
    if matches!(config.trim_mode, TrimMode::None) {
        return;
    }
    let rgba = match sprite.image.as_rgba8() {
        Some(img) => img,
        None => return,
    };
    let (img_w, img_h) = rgba.dimensions();
    let Some((mut x1, mut y1, mut x2, mut y2)) = find_opaque_bounds(rgba, config.trim_threshold)
    else {
        // Fully transparent — pack the full image with no trim offset.
        return;
    };

    // Expand by trim_margin, clamped to image bounds.
    let m = config.trim_margin;
    x1 = x1.saturating_sub(m);
    y1 = y1.saturating_sub(m);
    x2 = (x2 + m).min(img_w - 1);
    y2 = (y2 + m).min(img_h - 1);

    // Align width and height to common_divisor requirements.
    let tw = expand_to_divisor(x2 - x1 + 1, config.common_divisor_x).min(img_w - x1);
    let th = expand_to_divisor(y2 - y1 + 1, config.common_divisor_y).min(img_h - y1);

    sprite.trim_rect = Some(SourceRect {
        x: x1 as i32,
        y: y1 as i32,
        w: tw,
        h: th,
    });

    let cropped = image::imageops::crop_imm(rgba, x1, y1, tw, th).to_image();
    // Rehash with the trimmed pixels so alias detection compares the actual
    // packed content, not the (possibly differently-bordered) original.
    let mut h = FxHasher::default();
    h.write(cropped.as_raw());
    sprite.content_hash = h.finish();
    sprite.image = DynamicImage::ImageRgba8(cropped);
}

/// Return the inclusive pixel bounding box `(x_min, y_min, x_max, y_max)` of all
/// pixels whose alpha channel exceeds `threshold`. Returns `None` if the image is
/// entirely transparent at or below the threshold.
fn find_opaque_bounds(img: &image::RgbaImage, threshold: u8) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = img.dimensions();
    let mut x1 = u32::MAX;
    let mut y1 = u32::MAX;
    let mut x2 = 0u32;
    let mut y2 = 0u32;
    let mut found = false;

    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > threshold {
                if !found {
                    x1 = x;
                    y1 = y;
                    x2 = x;
                    y2 = y;
                    found = true;
                } else {
                    if x < x1 {
                        x1 = x;
                    }
                    if x > x2 {
                        x2 = x;
                    }
                    if y > y2 {
                        y2 = y;
                    }
                    // y1 cannot decrease since we scan top-to-bottom.
                }
            }
        }
    }

    if found { Some((x1, y1, x2, y2)) } else { None }
}

/// Round `size` up to the nearest multiple of `divisor`. Returns `size` unchanged
/// when `divisor` is 0 or 1.
fn expand_to_divisor(size: u32, divisor: u32) -> u32 {
    if divisor <= 1 {
        return size;
    }
    let r = size % divisor;
    if r == 0 { size } else { size + (divisor - r) }
}
