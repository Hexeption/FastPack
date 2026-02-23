use std::path::PathBuf;

use image::DynamicImage;
use serde::{Deserialize, Serialize};

use super::rect::{Point, Size, SourceRect};

/// Border widths for a 9-patch / 9-slice scalable sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NinePatch {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

/// A loaded, pre-processed sprite ready to be fed into a packing algorithm.
///
/// `image` contains pixel data *after* trimming and extrusion. Use
/// `original_size` and `trim_rect` to reconstruct the sprite's original position.
#[derive(Debug)]
pub struct Sprite {
    /// Unique identifier — typically the relative source path without extension
    /// (e.g. `"ui/button"`).
    pub id: String,

    /// Absolute path of the source image on disk.
    pub source_path: PathBuf,

    /// Pixel data ready for atlas composition (after trim + extrude).
    pub image: DynamicImage,

    /// Bounding box of the non-transparent region within the original image.
    /// `None` when `TrimMode::None` is in effect.
    pub trim_rect: Option<SourceRect>,

    /// Full dimensions of the original source image before any trimming.
    pub original_size: Size,

    /// Convex hull polygon vertices in image-space pixels.
    /// Only populated when `TrimMode::Polygon` is active.
    pub polygon: Option<Vec<Point>>,

    /// 9-patch border definition.
    pub nine_patch: Option<NinePatch>,

    /// Pivot point (0.0–1.0 normalised). `None` uses the project default.
    pub pivot: Option<Point>,

    /// xxHash64 of the trimmed RGBA pixel bytes, used for alias detection.
    pub content_hash: u64,

    /// If `Some(id)`, this sprite is a duplicate of the named one and will
    /// share its atlas rect rather than being placed independently.
    pub alias_of: Option<String>,
}
