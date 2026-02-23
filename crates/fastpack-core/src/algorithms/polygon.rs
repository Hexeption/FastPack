use geo::{ConvexHull, MultiPoint, Point as GeoPoint};
use image::RgbaImage;

use crate::types::rect::Point;

/// Compute the convex hull of all opaque pixels in `img`.
///
/// Vertices are in image-local pixel space (origin at the top-left of `img`).
/// Returns an empty `Vec` when no pixels exceed `threshold`.
pub fn compute_convex_hull(img: &RgbaImage, threshold: u8) -> Vec<Point> {
    let geo_points: Vec<GeoPoint<f64>> = img
        .enumerate_pixels()
        .filter(|(_, _, pixel)| pixel[3] > threshold)
        .map(|(x, y, _)| GeoPoint::new(x as f64, y as f64))
        .collect();

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
