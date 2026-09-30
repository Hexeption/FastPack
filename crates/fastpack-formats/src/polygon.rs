//! Polygon mesh data for frames packed with `TrimMode::Polygon`.
//!
//! Produces the `vertices` / `verticesUV` / `triangles` triple that
//! TexturePacker writes for polygon sprites in its JSON formats.

use fastpack_core::types::{atlas::AtlasFrame, rect::Point};
use serde::Serialize;

/// Triangulated polygon mesh for one frame, in TexturePacker's JSON layout.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolygonMesh {
    /// Hull vertices in source-sprite pixel space (origin at the top-left of
    /// the untrimmed image, i.e. the same space as `spriteSourceSize`).
    pub vertices: Vec<[i64; 2]>,
    /// The same vertices in atlas-texture pixel space.
    #[serde(rename = "verticesUV")]
    pub vertices_uv: Vec<[i64; 2]>,
    /// Vertex index triples.
    pub triangles: Vec<[usize; 3]>,
}

/// Build the mesh for `frame`, or `None` when the frame has no polygon.
///
/// `frame.polygon` holds a convex hull in trimmed-sprite space (before
/// extrusion and rotation). Atlas coordinates add the extrude border and
/// apply the 90° clockwise rotation the compositor uses, then offset by the
/// frame position. The hull is convex, so a triangle fan from vertex 0 is a
/// valid triangulation.
pub fn build_mesh(frame: &AtlasFrame) -> Option<PolygonMesh> {
    let hull = frame.polygon.as_deref()?;
    let hull = open_ring(hull);
    if hull.is_empty() {
        return None;
    }

    let e = frame.extrude as f32;
    let fx = frame.frame.x as f32;
    let fy = frame.frame.y as f32;
    // Height of the extruded image before rotation; after a clockwise
    // rotation it becomes the placed width.
    let unrotated_h = frame.frame.w as f32;
    let ox = frame.sprite_source_size.x as f32;
    let oy = frame.sprite_source_size.y as f32;

    let vertices = hull
        .iter()
        .map(|p| [round(p.x + ox), round(p.y + oy)])
        .collect();
    let vertices_uv = hull
        .iter()
        .map(|p| {
            let (x, y) = (p.x + e, p.y + e);
            if frame.rotated {
                [round(fx + unrotated_h - y), round(fy + x)]
            } else {
                [round(fx + x), round(fy + y)]
            }
        })
        .collect();
    let triangles = (1..hull.len().saturating_sub(1))
        .map(|i| [0, i, i + 1])
        .collect();

    Some(PolygonMesh {
        vertices,
        vertices_uv,
        triangles,
    })
}

/// Drop the closing vertex when the ring repeats its first point at the end.
fn open_ring(ring: &[Point]) -> &[Point] {
    match (ring.first(), ring.last()) {
        (Some(a), Some(b)) if ring.len() > 1 && a.x == b.x && a.y == b.y => &ring[..ring.len() - 1],
        _ => ring,
    }
}

fn round(v: f32) -> i64 {
    v.round() as i64
}
