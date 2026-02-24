use image::{DynamicImage, Rgba, RgbaImage};

use crate::types::pixel_format::PixelFormat;

/// Apply Floyd-Steinberg dithering to quantize an RGBA8888 image to the
/// target pixel format. The output is always RGBA8888; the quantization
/// reduces the number of unique colour values to match the target bit depth.
///
/// `PixelFormat::Rgba8888` is a no-op; the input is returned unchanged.
pub fn dither(image: &DynamicImage, format: PixelFormat) -> DynamicImage {
    match format {
        PixelFormat::Rgba8888 => image.clone(),
        PixelFormat::Rgb888 => dither_rgb888(image),
        PixelFormat::Rgb565 => dither_rgb565(image),
        PixelFormat::Rgba4444 => dither_rgba4444(image),
        PixelFormat::Rgba5551 => dither_rgba5551(image),
        PixelFormat::Alpha8 => dither_alpha8(image),
    }
}

fn dither_rgb565(image: &DynamicImage) -> DynamicImage {
    let src = image.to_rgba8();
    let (w, h) = src.dimensions();
    let mut err = vec![[0i32; 4]; (w * h) as usize];

    let mut dst = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = src.get_pixel(x, y);
            let idx = (y * w + x) as usize;

            let r_in = (p[0] as i32 + err[idx][0]).clamp(0, 255) as u8;
            let g_in = (p[1] as i32 + err[idx][1]).clamp(0, 255) as u8;
            let b_in = (p[2] as i32 + err[idx][2]).clamp(0, 255) as u8;

            let r_q = quantize5(r_in);
            let g_q = quantize6(g_in);
            let b_q = quantize5(b_in);

            dst.put_pixel(x, y, Rgba([r_q, g_q, b_q, 255]));

            diffuse(&mut err, w, h, x, y, r_in as i32 - r_q as i32, 0);
            diffuse(&mut err, w, h, x, y, g_in as i32 - g_q as i32, 1);
            diffuse(&mut err, w, h, x, y, b_in as i32 - b_q as i32, 2);
        }
    }
    DynamicImage::ImageRgba8(dst)
}

fn dither_rgb888(image: &DynamicImage) -> DynamicImage {
    // RGB888 is the same bit depth as RGBA8888 channels; just drop alpha.
    let src = image.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = src.get_pixel(x, y);
            dst.put_pixel(x, y, Rgba([p[0], p[1], p[2], 255]));
        }
    }
    DynamicImage::ImageRgba8(dst)
}

fn dither_rgba4444(image: &DynamicImage) -> DynamicImage {
    let src = image.to_rgba8();
    let (w, h) = src.dimensions();
    let mut err = vec![[0i32; 4]; (w * h) as usize];

    let mut dst = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = src.get_pixel(x, y);
            let idx = (y * w + x) as usize;

            let r_in = (p[0] as i32 + err[idx][0]).clamp(0, 255) as u8;
            let g_in = (p[1] as i32 + err[idx][1]).clamp(0, 255) as u8;
            let b_in = (p[2] as i32 + err[idx][2]).clamp(0, 255) as u8;
            let a_in = (p[3] as i32 + err[idx][3]).clamp(0, 255) as u8;

            let r_q = quantize4(r_in);
            let g_q = quantize4(g_in);
            let b_q = quantize4(b_in);
            let a_q = quantize4(a_in);

            dst.put_pixel(x, y, Rgba([r_q, g_q, b_q, a_q]));

            diffuse(&mut err, w, h, x, y, r_in as i32 - r_q as i32, 0);
            diffuse(&mut err, w, h, x, y, g_in as i32 - g_q as i32, 1);
            diffuse(&mut err, w, h, x, y, b_in as i32 - b_q as i32, 2);
            diffuse(&mut err, w, h, x, y, a_in as i32 - a_q as i32, 3);
        }
    }
    DynamicImage::ImageRgba8(dst)
}

fn dither_rgba5551(image: &DynamicImage) -> DynamicImage {
    let src = image.to_rgba8();
    let (w, h) = src.dimensions();
    let mut err = vec![[0i32; 4]; (w * h) as usize];

    let mut dst = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = src.get_pixel(x, y);
            let idx = (y * w + x) as usize;

            let r_in = (p[0] as i32 + err[idx][0]).clamp(0, 255) as u8;
            let g_in = (p[1] as i32 + err[idx][1]).clamp(0, 255) as u8;
            let b_in = (p[2] as i32 + err[idx][2]).clamp(0, 255) as u8;
            let a_in = (p[3] as i32 + err[idx][3]).clamp(0, 255) as u8;

            let r_q = quantize5(r_in);
            let g_q = quantize5(g_in);
            let b_q = quantize5(b_in);
            let a_q = if a_in >= 128 { 255u8 } else { 0u8 };

            dst.put_pixel(x, y, Rgba([r_q, g_q, b_q, a_q]));

            diffuse(&mut err, w, h, x, y, r_in as i32 - r_q as i32, 0);
            diffuse(&mut err, w, h, x, y, g_in as i32 - g_q as i32, 1);
            diffuse(&mut err, w, h, x, y, b_in as i32 - b_q as i32, 2);
            diffuse(&mut err, w, h, x, y, a_in as i32 - a_q as i32, 3);
        }
    }
    DynamicImage::ImageRgba8(dst)
}

fn dither_alpha8(image: &DynamicImage) -> DynamicImage {
    // Alpha-only: keep alpha, set RGB to zero.
    let src = image.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = src.get_pixel(x, y);
            dst.put_pixel(x, y, Rgba([0, 0, 0, p[3]]));
        }
    }
    DynamicImage::ImageRgba8(dst)
}

/// Floyd-Steinberg error diffusion. Spreads `error` in channel `ch` from
/// pixel (x, y) to its four neighbours using the standard 7/16, 3/16,
/// 5/16, 1/16 weights.
fn diffuse(err: &mut [[i32; 4]], w: u32, h: u32, x: u32, y: u32, error: i32, ch: usize) {
    if error == 0 {
        return;
    }
    let (x, y, w, h) = (x as usize, y as usize, w as usize, h as usize);

    if x + 1 < w {
        err[y * w + (x + 1)][ch] += error * 7 / 16;
    }
    if y + 1 < h {
        if x > 0 {
            err[(y + 1) * w + (x - 1)][ch] += error * 3 / 16;
        }
        err[(y + 1) * w + x][ch] += error * 5 / 16;
        if x + 1 < w {
            err[(y + 1) * w + (x + 1)][ch] += error / 16;
        }
    }
}

/// Quantize an 8-bit value to 5-bit precision, then expand back to 8 bits.
/// Expansion uses the standard replication formula: (v5 << 3) | (v5 >> 2).
fn quantize5(v: u8) -> u8 {
    let v5 = v >> 3;
    (v5 << 3) | (v5 >> 2)
}

/// Quantize an 8-bit value to 6-bit precision, then expand back to 8 bits.
fn quantize6(v: u8) -> u8 {
    let v6 = v >> 2;
    (v6 << 2) | (v6 >> 4)
}

/// Quantize an 8-bit value to 4-bit precision, then expand back to 8 bits.
/// 4-bit value expands as (v4 << 4) | v4 = v4 * 17.
fn quantize4(v: u8) -> u8 {
    let v4 = v >> 4;
    (v4 << 4) | v4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba8888_is_noop() {
        let img = DynamicImage::new_rgba8(4, 4);
        let out = dither(&img, PixelFormat::Rgba8888);
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 4);
    }

    #[test]
    fn rgb565_alpha_forced_to_255() {
        let mut src = RgbaImage::new(1, 1);
        src.put_pixel(0, 0, Rgba([255, 128, 64, 200]));
        let out = dither(&DynamicImage::ImageRgba8(src), PixelFormat::Rgb565);
        let p = out.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(p[3], 255, "rgb565 forces alpha to 255");
    }

    #[test]
    fn rgba5551_threshold_opaque() {
        let mut src = RgbaImage::new(1, 1);
        src.put_pixel(0, 0, Rgba([255, 255, 255, 200]));
        let out = dither(&DynamicImage::ImageRgba8(src), PixelFormat::Rgba5551);
        let p = out.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(p[3], 255, "alpha >= 128 rounds to 255");
    }

    #[test]
    fn rgba5551_threshold_transparent() {
        let mut src = RgbaImage::new(1, 1);
        src.put_pixel(0, 0, Rgba([255, 255, 255, 50]));
        let out = dither(&DynamicImage::ImageRgba8(src), PixelFormat::Rgba5551);
        let p = out.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(p[3], 0, "alpha < 128 rounds to 0");
    }

    #[test]
    fn quantize5_round_trips() {
        // Pure white should survive round-trip.
        assert_eq!(quantize5(255), 255);
        // Pure black should survive.
        assert_eq!(quantize5(0), 0);
    }

    #[test]
    fn quantize4_round_trips() {
        assert_eq!(quantize4(255), 255);
        assert_eq!(quantize4(0), 0);
    }

    #[test]
    fn rgba4444_preserves_dimensions() {
        let img = DynamicImage::new_rgba8(8, 8);
        let out = dither(&img, PixelFormat::Rgba4444);
        assert_eq!((out.width(), out.height()), (8, 8));
    }
}
