use image::{DynamicImage, Rgba, RgbaImage};

/// Premultiply the alpha channel into the RGB channels.
///
/// Each pixel becomes `(R * A / 255, G * A / 255, B * A / 255, A)`.
/// Fully transparent pixels (A == 0) are set to `(0, 0, 0, 0)`.
///
/// Premultiplied alpha is required by some rendering APIs (CoreGraphics,
/// certain WebGL blend modes) and produces better filtering results when
/// mipmaps are generated.
pub fn premultiply(image: &DynamicImage) -> DynamicImage {
    let src = image.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = src.get_pixel(x, y);
            let a = p[3];
            if a == 255 {
                dst.put_pixel(x, y, *p);
            } else if a == 0 {
                dst.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            } else {
                dst.put_pixel(
                    x,
                    y,
                    Rgba([
                        (p[0] as u16 * a as u16 / 255) as u8,
                        (p[1] as u16 * a as u16 / 255) as u8,
                        (p[2] as u16 * a as u16 / 255) as u8,
                        a,
                    ]),
                );
            }
        }
    }
    DynamicImage::ImageRgba8(dst)
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, Rgba, RgbaImage};

    use super::*;

    #[test]
    fn opaque_pixel_unchanged() {
        let mut src = RgbaImage::new(1, 1);
        src.put_pixel(0, 0, Rgba([200, 100, 50, 255]));
        let out = premultiply(&DynamicImage::ImageRgba8(src));
        let p = out.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(p, [200, 100, 50, 255]);
    }

    #[test]
    fn transparent_pixel_zeroed() {
        let mut src = RgbaImage::new(1, 1);
        src.put_pixel(0, 0, Rgba([255, 255, 255, 0]));
        let out = premultiply(&DynamicImage::ImageRgba8(src));
        let p = out.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(p, [0, 0, 0, 0]);
    }

    #[test]
    fn half_alpha_halves_rgb() {
        let mut src = RgbaImage::new(1, 1);
        src.put_pixel(0, 0, Rgba([200, 100, 0, 128]));
        let out = premultiply(&DynamicImage::ImageRgba8(src));
        let p = out.to_rgba8().get_pixel(0, 0).0;
        // 200 * 128 / 255 ≈ 100, within 1
        assert!((p[0] as i32 - 100).abs() <= 1, "R ≈ 100, got {}", p[0]);
        // 100 * 128 / 255 ≈ 50, within 1
        assert!((p[1] as i32 - 50).abs() <= 1, "G ≈ 50, got {}", p[1]);
        assert_eq!(p[2], 0);
        assert_eq!(p[3], 128);
    }

    #[test]
    fn alpha_channel_preserved() {
        let mut src = RgbaImage::new(1, 1);
        src.put_pixel(0, 0, Rgba([100, 100, 100, 77]));
        let out = premultiply(&DynamicImage::ImageRgba8(src));
        let p = out.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(p[3], 77);
    }
}
