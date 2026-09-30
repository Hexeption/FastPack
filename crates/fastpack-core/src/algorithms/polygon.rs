use geo::{ConvexHull, MultiPoint, Point as GeoPoint};
use image::RgbaImage;

use crate::types::rect::Point;

/// Compute the convex hull of all opaque pixels in `img`.
///
/// Each opaque pixel is treated as the unit square it covers, so the hull
/// encloses whole pixels: a fully opaque `w×h` image yields the rectangle
/// `(0,0)–(w,h)`. Vertices are in image-local pixel space (origin at the
/// top-left of `img`). The returned ring is closed (the first vertex is
/// repeated at the end). Returns an empty `Vec` when no pixels exceed
/// `threshold`.
pub fn compute_convex_hull(img: &RgbaImage, threshold: u8) -> Vec<Point> {
    let (w, h) = img.dimensions();
    let mut geo_points: Vec<GeoPoint<f64>> = Vec::new();

    // Only the leftmost and rightmost opaque pixel of each row can contribute
    // hull vertices, so emit just their outer corners.
    for y in 0..h {
        let mut left = None;
        let mut right = 0u32;
        for x in 0..w {
            if img.get_pixel(x, y)[3] > threshold {
                if left.is_none() {
                    left = Some(x);
                }
                right = x;
            }
        }
        if let Some(left) = left {
            let (x0, x1) = (left as f64, (right + 1) as f64);
            let (y0, y1) = (y as f64, (y + 1) as f64);
            geo_points.extend([
                GeoPoint::new(x0, y0),
                GeoPoint::new(x0, y1),
                GeoPoint::new(x1, y0),
                GeoPoint::new(x1, y1),
            ]);
        }
    }

    if geo_points.is_empty() {
        return Vec::new();
    }

    let hull = MultiPoint(geo_points).convex_hull();
    hull.exterior()
        .coords()
        .map(|c| Point {
            x: c.x as f32,
            y: c.y as f32,
        })
        .collect()
}
