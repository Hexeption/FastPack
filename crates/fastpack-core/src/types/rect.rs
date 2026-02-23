use serde::{Deserialize, Serialize};

/// Integer rectangle used for atlas placement coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    #[inline]
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    #[inline]
    pub fn area(&self) -> u32 {
        self.w * self.h
    }

    #[inline]
    pub fn right(&self) -> u32 {
        self.x + self.w
    }

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
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// Normalised 2-D point (typically 0.0–1.0 for pivot coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
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
    pub w: u32,
    pub h: u32,
}

impl Size {
    #[inline]
    pub fn area(&self) -> u32 {
        self.w * self.h
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
}
