use image::{DynamicImage, GenericImage, GenericImageView, RgbaImage};

use crate::types::sprite::Sprite;

/// Extrude a sprite's image by repeating its border pixels outward.
///
/// Creates a new image of size `(w + 2*amount) × (h + 2*amount)` where the
/// original image is centred and each border is filled by clamping to the
/// nearest source pixel. This prevents texture bleeding when hardware filtering
/// samples outside the intended sprite boundary at atlas seams.
///
/// `sprite.extrude` is updated to record the applied amount. The function is a
/// no-op when `amount` is zero or the image is not RGBA8.
pub fn extrude(sprite: &mut Sprite, amount: u32) {
    if amount == 0 {
        return;
    }

    let out = {
        let rgba = match sprite.image.as_rgba8() {
            Some(img) => img,
            None => return,
        };
        let (w, h) = rgba.dimensions();
        if w == 0 || h == 0 {
            return;
        }

        let aw = w + 2 * amount;
        let ah = h + 2 * amount;
        let mut out = RgbaImage::new(aw, ah);

        // For each output pixel, sample the source at the nearest clamped position.
        // This handles all four borders and all four corners in a single pass.
        for oy in 0..ah {
            let sy = oy.saturating_sub(amount).min(h - 1);
            for ox in 0..aw {
                let sx = ox.saturating_sub(amount).min(w - 1);
                out.put_pixel(ox, oy, *rgba.get_pixel(sx, sy));
            }
        }

        out
    };

    sprite.extrude = amount;
    sprite.image = DynamicImage::ImageRgba8(out);
}
