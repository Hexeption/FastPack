use image::RgbaImage;

use crate::types::sprite::Sprite;

/// Fill the RGB channels of fully transparent pixels with the colour of their
/// nearest visible neighbours ("alpha bleeding").
///
/// Bilinear filtering and mipmapping blend fully transparent texels with their
/// visible neighbours. Transparent pixels usually store black (or arbitrary)
/// RGB, which produces dark fringes around sprite edges. This pass floods the
/// colour of every pixel with `alpha > 0` outward into the transparent area,
/// ring by ring, so the hidden RGB matches the adjacent visible colour. The
/// alpha channel is never modified, so the sprite looks identical when drawn
/// without filtering.
///
/// Each transparent pixel takes the average RGB of its already-coloured
/// 8-neighbours from the previous ring. The pass is a breadth-first flood and
/// visits every pixel a constant number of times, so it runs in `O(w * h)`.
///
/// The function is a no-op when the image is not RGBA8, is empty, or contains
/// no visible pixels.
pub fn alpha_bleed(sprite: &mut Sprite) {
    if let Some(img) = sprite.image.as_mut_rgba8() {
        bleed_image(img);
    }
}

/// Apply alpha bleeding to an RGBA8 buffer in place. See [`alpha_bleed`].
pub fn bleed_image(img: &mut RgbaImage) {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return;
    }
    let (w, h) = (w as usize, h as usize);
    let buf: &mut [u8] = img.as_mut();

    // `done[i]` is true once pixel `i` holds a valid colour: either it was
    // visible to begin with, or it was filled by an earlier ring.
    let mut done: Vec<bool> = buf.chunks_exact(4).map(|p| p[3] != 0).collect();

    // Offsets for the 8-neighbourhood.
    const NEIGHBOURS: [(isize, isize); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let neighbours = |i: usize| {
        let x = (i % w) as isize;
        let y = (i / w) as isize;
        NEIGHBOURS.iter().filter_map(move |&(dx, dy)| {
            let nx = x + dx;
            let ny = y + dy;
            (nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h)
                .then(|| ny as usize * w + nx as usize)
        })
    };

    // Seed the first ring: transparent pixels touching a visible pixel.
    // `queued` prevents a pixel from entering the frontier twice.
    let mut queued = vec![false; w * h];
    let mut frontier: Vec<usize> = Vec::new();
    for i in 0..w * h {
        if !done[i] && neighbours(i).any(|n| done[n]) {
            queued[i] = true;
            frontier.push(i);
        }
    }

    let mut colours: Vec<[u8; 3]> = Vec::new();
    let mut next: Vec<usize> = Vec::new();
    while !frontier.is_empty() {
        // Compute every colour of the ring before committing any of them, so
        // pixels in the same ring do not influence each other.
        colours.clear();
        for &i in &frontier {
            let (mut r, mut g, mut b, mut n) = (0u32, 0u32, 0u32, 0u32);
            for j in neighbours(i) {
                if done[j] {
                    let p = &buf[j * 4..j * 4 + 3];
                    r += p[0] as u32;
                    g += p[1] as u32;
                    b += p[2] as u32;
                    n += 1;
                }
            }
            // Every frontier pixel has at least one coloured neighbour.
            colours.push([
                ((r + n / 2) / n) as u8,
                ((g + n / 2) / n) as u8,
                ((b + n / 2) / n) as u8,
            ]);
        }

        next.clear();
        for (&i, c) in frontier.iter().zip(&colours) {
            buf[i * 4..i * 4 + 3].copy_from_slice(c);
            done[i] = true;
        }
        for &i in &frontier {
            for j in neighbours(i) {
                if !done[j] && !queued[j] {
                    queued[j] = true;
                    next.push(j);
                }
            }
        }
        std::mem::swap(&mut frontier, &mut next);
    }
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, Rgba, RgbaImage};

    use super::*;

    fn sprite_from(img: RgbaImage) -> Sprite {
        let (w, h) = img.dimensions();
        Sprite {
            id: "s".into(),
            source_path: Default::default(),
            image: DynamicImage::ImageRgba8(img),
            trim_rect: None,
            original_size: crate::types::rect::Size { w, h },
            polygon: None,
            pivot: None,
            nine_patch: None,
            content_hash: 0,
            extrude: 0,
            alias_of: None,
        }
    }

    #[test]
    fn fills_transparent_pixels_with_neighbour_colour() {
        let mut img = RgbaImage::new(5, 5);
        img.put_pixel(2, 2, Rgba([200, 100, 50, 255]));
        bleed_image(&mut img);
        for (x, y, p) in img.enumerate_pixels() {
            if (x, y) == (2, 2) {
                assert_eq!(p.0, [200, 100, 50, 255]);
            } else {
                assert_eq!(p.0, [200, 100, 50, 0], "pixel ({x},{y})");
            }
        }
    }

    #[test]
    fn alpha_channel_is_never_modified() {
        let mut img = RgbaImage::new(4, 4);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 128]));
        img.put_pixel(3, 3, Rgba([0, 0, 255, 255]));
        let before: Vec<u8> = img.pixels().map(|p| p[3]).collect();
        bleed_image(&mut img);
        let after: Vec<u8> = img.pixels().map(|p| p[3]).collect();
        assert_eq!(before, after);
    }

    #[test]
    fn visible_pixels_are_unchanged() {
        let mut img = RgbaImage::new(3, 1);
        img.put_pixel(0, 0, Rgba([10, 20, 30, 1]));
        img.put_pixel(2, 0, Rgba([40, 50, 60, 255]));
        bleed_image(&mut img);
        assert_eq!(img.get_pixel(0, 0).0, [10, 20, 30, 1]);
        assert_eq!(img.get_pixel(2, 0).0, [40, 50, 60, 255]);
    }

    #[test]
    fn middle_pixel_averages_two_sources() {
        let mut img = RgbaImage::new(3, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 255]));
        img.put_pixel(2, 0, Rgba([200, 100, 50, 255]));
        bleed_image(&mut img);
        assert_eq!(img.get_pixel(1, 0).0, [100, 50, 25, 0]);
    }

    #[test]
    fn nearest_source_wins() {
        // Red on the left, blue on the right; each side takes its nearest colour.
        let mut img = RgbaImage::new(7, 1);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        img.put_pixel(6, 0, Rgba([0, 0, 255, 255]));
        bleed_image(&mut img);
        assert_eq!(img.get_pixel(1, 0).0, [255, 0, 0, 0]);
        assert_eq!(img.get_pixel(2, 0).0, [255, 0, 0, 0]);
        assert_eq!(img.get_pixel(4, 0).0, [0, 0, 255, 0]);
        assert_eq!(img.get_pixel(5, 0).0, [0, 0, 255, 0]);
    }

    #[test]
    fn fully_transparent_image_is_noop() {
        let mut img = RgbaImage::from_pixel(3, 3, Rgba([7, 8, 9, 0]));
        bleed_image(&mut img);
        assert!(img.pixels().all(|p| p.0 == [7, 8, 9, 0]));
    }

    #[test]
    fn empty_image_is_noop() {
        let mut img = RgbaImage::new(0, 0);
        bleed_image(&mut img);
        assert_eq!(img.dimensions(), (0, 0));
    }

    #[test]
    fn sprite_wrapper_bleeds_rgba8() {
        let mut img = RgbaImage::new(2, 1);
        img.put_pixel(0, 0, Rgba([9, 9, 9, 255]));
        let mut sprite = sprite_from(img);
        alpha_bleed(&mut sprite);
        let out = sprite.image.as_rgba8().unwrap();
        assert_eq!(out.get_pixel(1, 0).0, [9, 9, 9, 0]);
    }
}
