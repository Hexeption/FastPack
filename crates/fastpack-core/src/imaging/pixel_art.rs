use image::{Rgba, RgbaImage};

/// Scale2x / EPX algorithm. Produces a 2× pixel-art upscale of `src`.
///
/// Each source pixel becomes a 2×2 block. The four output pixels are chosen
/// by comparing the four cardinal neighbours of the source pixel.
pub fn scale2x(src: &RgbaImage) -> RgbaImage {
    let (img_w, img_h) = src.dimensions();
    let mut dst = RgbaImage::new(img_w * 2, img_h * 2);

    for row in 0..img_h {
        for col in 0..img_w {
            let p = *src.get_pixel(col, row);
            let n = if row > 0 {
                *src.get_pixel(col, row - 1)
            } else {
                p
            };
            let w = if col > 0 {
                *src.get_pixel(col - 1, row)
            } else {
                p
            };
            let e = if col < img_w - 1 {
                *src.get_pixel(col + 1, row)
            } else {
                p
            };
            let s = if row < img_h - 1 {
                *src.get_pixel(col, row + 1)
            } else {
                p
            };

            // EPX rules: each corner picks the adjacent cardinal pixel when two
            // matching neighbours form a straight edge and the opposite pair does not.
            let e0 = if w == n && w != s && n != e { w } else { p }; // top-left
            let e1 = if n == e && n != w && e != s { e } else { p }; // top-right
            let e2 = if w == s && w != n && s != e { w } else { p }; // bottom-left
            let e3 = if s == e && w != s && n != e { e } else { p }; // bottom-right

            dst.put_pixel(col * 2, row * 2, e0);
            dst.put_pixel(col * 2 + 1, row * 2, e1);
            dst.put_pixel(col * 2, row * 2 + 1, e2);
            dst.put_pixel(col * 2 + 1, row * 2 + 1, e3);
        }
    }
    dst
}

/// Scale3x algorithm. Produces a 3× pixel-art upscale of `src`.
///
/// Each source pixel becomes a 3×3 block. The output pixels are determined by
/// comparing all 8 neighbours of the source pixel.
pub fn scale3x(src: &RgbaImage) -> RgbaImage {
    let (img_w, img_h) = src.dimensions();
    let mut dst = RgbaImage::new(img_w * 3, img_h * 3);

    for row in 0..img_h {
        for col in 0..img_w {
            let e = *src.get_pixel(col, row);

            let nw = if row > 0 && col > 0 {
                *src.get_pixel(col - 1, row - 1)
            } else {
                e
            };
            let n = if row > 0 {
                *src.get_pixel(col, row - 1)
            } else {
                e
            };
            let ne = if row > 0 && col < img_w - 1 {
                *src.get_pixel(col + 1, row - 1)
            } else {
                e
            };
            let w = if col > 0 {
                *src.get_pixel(col - 1, row)
            } else {
                e
            };
            let ea = if col < img_w - 1 {
                *src.get_pixel(col + 1, row)
            } else {
                e
            };
            let sw = if row < img_h - 1 && col > 0 {
                *src.get_pixel(col - 1, row + 1)
            } else {
                e
            };
            let s = if row < img_h - 1 {
                *src.get_pixel(col, row + 1)
            } else {
                e
            };
            let se = if row < img_h - 1 && col < img_w - 1 {
                *src.get_pixel(col + 1, row + 1)
            } else {
                e
            };

            let r0 = if w == n && n != ea && w != s { w } else { e };
            let r1 = if (w == n && n != ea && w != s && e != ne)
                || (n == ea && n != w && ea != s && e != nw)
            {
                n
            } else {
                e
            };
            let r2 = if n == ea && n != w && ea != s { ea } else { e };
            let r3 = if (w == n && n != ea && w != s && e != sw)
                || (w == s && w != n && s != ea && e != nw)
            {
                w
            } else {
                e
            };
            let r4 = e;
            let r5 = if (n == ea && n != w && ea != s && e != se)
                || (s == ea && w != s && n != ea && e != ne)
            {
                ea
            } else {
                e
            };
            let r6 = if w == s && w != n && s != ea { w } else { e };
            let r7 = if (w == s && w != n && s != ea && e != se)
                || (s == ea && w != s && n != ea && e != sw)
            {
                s
            } else {
                e
            };
            let r8 = if s == ea && w != s && n != ea { ea } else { e };

            let ox = col * 3;
            let oy = row * 3;
            dst.put_pixel(ox, oy, r0);
            dst.put_pixel(ox + 1, oy, r1);
            dst.put_pixel(ox + 2, oy, r2);
            dst.put_pixel(ox, oy + 1, r3);
            dst.put_pixel(ox + 1, oy + 1, r4);
            dst.put_pixel(ox + 2, oy + 1, r5);
            dst.put_pixel(ox, oy + 2, r6);
            dst.put_pixel(ox + 1, oy + 2, r7);
            dst.put_pixel(ox + 2, oy + 2, r8);
        }
    }
    dst
}

/// Eagle 2× algorithm. Produces a 2× pixel-art upscale of `src`.
///
/// Each corner of the 2×2 output block takes the colour of the matching
/// diagonal neighbour when that neighbour also matches both adjacent cardinal
/// neighbours on the same side. Otherwise the source pixel is kept.
pub fn eagle2x(src: &RgbaImage) -> RgbaImage {
    let (img_w, img_h) = src.dimensions();
    let mut dst = RgbaImage::new(img_w * 2, img_h * 2);

    for row in 0..img_h {
        for col in 0..img_w {
            let p = *src.get_pixel(col, row);
            let n = if row > 0 {
                *src.get_pixel(col, row - 1)
            } else {
                p
            };
            let w = if col > 0 {
                *src.get_pixel(col - 1, row)
            } else {
                p
            };
            let e = if col < img_w - 1 {
                *src.get_pixel(col + 1, row)
            } else {
                p
            };
            let s = if row < img_h - 1 {
                *src.get_pixel(col, row + 1)
            } else {
                p
            };
            let nw = if row > 0 && col > 0 {
                *src.get_pixel(col - 1, row - 1)
            } else {
                p
            };
            let ne = if row > 0 && col < img_w - 1 {
                *src.get_pixel(col + 1, row - 1)
            } else {
                p
            };
            let sw = if row < img_h - 1 && col > 0 {
                *src.get_pixel(col - 1, row + 1)
            } else {
                p
            };
            let se = if row < img_h - 1 && col < img_w - 1 {
                *src.get_pixel(col + 1, row + 1)
            } else {
                p
            };

            let e0 = if nw == n && nw == w { nw } else { p }; // top-left
            let e1 = if ne == n && ne == e { ne } else { p }; // top-right
            let e2 = if sw == s && sw == w { sw } else { p }; // bottom-left
            let e3 = if se == s && se == e { se } else { p }; // bottom-right

            dst.put_pixel(col * 2, row * 2, e0);
            dst.put_pixel(col * 2 + 1, row * 2, e1);
            dst.put_pixel(col * 2, row * 2 + 1, e2);
            dst.put_pixel(col * 2 + 1, row * 2 + 1, e3);
        }
    }
    dst
}

/// Simplified HQ2x-style scaler. Produces a 2× output from `src`.
///
/// This is not the full 256-entry LUT HQ2x but uses the same core idea:
/// each corner pixel of the 2×2 output block blends towards its two adjacent
/// cardinal neighbours when those neighbours share the same colour and that
/// colour differs from the centre pixel. This smooths diagonal edges in pixel
/// art without introducing blurring on horizontal or vertical edges.
pub fn hq2x(src: &RgbaImage) -> RgbaImage {
    let (img_w, img_h) = src.dimensions();
    let mut dst = RgbaImage::new(img_w * 2, img_h * 2);

    for row in 0..img_h {
        for col in 0..img_w {
            let p = *src.get_pixel(col, row);
            let n = if row > 0 {
                *src.get_pixel(col, row - 1)
            } else {
                p
            };
            let w = if col > 0 {
                *src.get_pixel(col - 1, row)
            } else {
                p
            };
            let e = if col < img_w - 1 {
                *src.get_pixel(col + 1, row)
            } else {
                p
            };
            let s = if row < img_h - 1 {
                *src.get_pixel(col, row + 1)
            } else {
                p
            };

            // Each corner blends when the two adjacent cardinal neighbours match
            // and differ from the centre pixel.
            let p0 = if n == w && n != p { blend3(p, n, w) } else { p }; // top-left
            let p1 = if n == e && n != p { blend3(p, n, e) } else { p }; // top-right
            let p2 = if s == w && s != p { blend3(p, s, w) } else { p }; // bottom-left
            let p3 = if s == e && s != p { blend3(p, s, e) } else { p }; // bottom-right

            dst.put_pixel(col * 2, row * 2, p0);
            dst.put_pixel(col * 2 + 1, row * 2, p1);
            dst.put_pixel(col * 2, row * 2 + 1, p2);
            dst.put_pixel(col * 2 + 1, row * 2 + 1, p3);
        }
    }
    dst
}

/// Weighted average: (2×base + a + b) / 4 per channel.
fn blend3(base: Rgba<u8>, a: Rgba<u8>, b: Rgba<u8>) -> Rgba<u8> {
    Rgba([
        ((base[0] as u16 * 2 + a[0] as u16 + b[0] as u16) / 4) as u8,
        ((base[1] as u16 * 2 + a[1] as u16 + b[1] as u16) / 4) as u8,
        ((base[2] as u16 * 2 + a[2] as u16 + b[2] as u16) / 4) as u8,
        ((base[3] as u16 * 2 + a[3] as u16 + b[3] as u16) / 4) as u8,
    ])
}
