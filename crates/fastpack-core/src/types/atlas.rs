use image::DynamicImage;
use serde::{Deserialize, Serialize};

use super::rect::{Point, Rect, Size, SourceRect};
use super::sprite::NinePatch;

/// A single frame within a completed atlas texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasFrame {
    /// Sprite identifier (matches `Sprite::id`).
    pub id: String,

    /// Position and size within the atlas texture.
    pub frame: Rect,

    /// `true` if this sprite was rotated 90° clockwise during packing.
    pub rotated: bool,

    /// `true` if transparent borders were stripped from this sprite.
    pub trimmed: bool,

    /// Offset and size of the sprite content within the original image.
    pub sprite_source_size: SourceRect,

    /// Full dimensions of the original source image.
    pub source_size: Size,

    /// Convex hull polygon vertices (only present for `TrimMode::Polygon`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polygon: Option<Vec<Point>>,

    /// 9-patch border widths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nine_patch: Option<NinePatch>,

    /// Normalised pivot point.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pivot: Option<Point>,

    /// If present, this frame shares the pixel data of the named sprite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias_of: Option<String>,
}

/// The result of a single atlas packing pass.
pub struct PackedAtlas {
    /// All packed frames in insertion order.
    pub frames: Vec<AtlasFrame>,

    /// Final atlas texture dimensions.
    pub size: Size,

    /// Composited atlas image ready for encoding to disk.
    /// `None` only in `Basic` mode when no data file is requested.
    pub image: Option<DynamicImage>,

    /// Base output file name (without extension or scale suffix).
    pub name: String,

    /// Scale factor this atlas represents (1.0 = full resolution).
    pub scale: f32,
}

/// Multiple atlas sheets produced by a multipack operation.
pub type AtlasSet = Vec<PackedAtlas>;
