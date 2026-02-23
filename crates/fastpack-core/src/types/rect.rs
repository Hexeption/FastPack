use serde::{Deserialize, Serialize};

/// Integer rectangle used for atlas placement coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Rect {
    /// Left edge in pixels.
    pub x: u32,
    /// Top edge in pixels.
    pub y: u32,
    /// Width in pixels.
    pub w: u32,
    /// Height in pixels.
    pub h: u32,
}

impl Rect {
    /// Construct a rectangle from its left edge, top edge, width, and height.
    #[inline]
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    /// Area in pixels squared.
    #[inline]
    pub fn area(&self) -> u32 {
        self.w * self.h
    }

    /// One pixel past the right edge (`x + w`).
    #[inline]
    pub fn right(&self) -> u32 {
        self.x + self.w
    }

    /// One pixel past the bottom edge (`y + h`).
    #[inline]
    pub fn bottom(&self) -> u32 {
        self.y + self.h
    }

    /// `true` if `other` fits entirely within `self`.
    pub fn contains(&self, other: &Self) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.right() >= other.right()
            && self.bottom() >= other.bottom()
    }

    /// `true` if the two rectangles overlap (touching edges do not count).
    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }
}

/// Source region within the original image.
///
/// Coordinates may be negative for `CropKeepPos` mode, where the sprite has
/// transparent padding that must be preserved for correct registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceRect {
    /// Left offset within the original image (can be negative in `CropKeepPos` mode).
    pub x: i32,
    /// Top offset within the original image (can be negative in `CropKeepPos` mode).
    pub y: i32,
    /// Width of the region in pixels.
    pub w: u32,
    /// Height of the region in pixels.
    pub h: u32,
}

/// Normalised 2-D point (typically 0.0–1.0 for pivot coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// Horizontal position (0.0 = left, 1.0 = right).
    pub x: f32,
    /// Vertical position (0.0 = top, 1.0 = bottom).
    pub y: f32,
}

/// Default pivot is the centre (0.5, 0.5).
impl Default for Point {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 }
    }
}

/// Axis-aligned pixel dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Size {
    /// Width in pixels.
    pub w: u32,
    /// Height in pixels.
    pub h: u32,
}

impl Size {
    /// Area in pixels squared.
    #[inline]
    pub fn area(&self) -> u32 {
        self.w * self.h
    }

    /// `true` if either dimension is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
}
