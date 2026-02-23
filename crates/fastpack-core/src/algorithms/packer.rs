use crate::{
    error::CoreError,
    types::{
        config::{LayoutConfig, SpriteConfig},
        rect::{Rect, Size},
        sprite::Sprite,
    },
};

/// Input consumed by a packing algorithm.
pub struct PackInput {
    /// Pre-processed sprites (trimmed, extruded, scaled to target variant).
    pub sprites: Vec<Sprite>,
    /// Atlas layout constraints.
    pub config: LayoutConfig,
    /// Sprite-level settings (e.g. common divisors).
    pub sprite_config: SpriteConfig,
}

/// Position of one sprite within the atlas.
pub struct Placement {
    /// Matches `Sprite::id`.
    pub sprite_id: String,
    /// Destination rectangle within the atlas texture.
    pub dest: Rect,
    /// `true` when the sprite was rotated 90° clockwise.
    pub rotated: bool,
}

/// A sprite that was successfully placed, paired with its destination in the atlas.
pub struct PlacedSprite {
    pub sprite: Sprite,
    pub placement: Placement,
}

/// Output produced by a single atlas packing pass.
pub struct PackOutput {
    /// Successfully placed sprites with their atlas positions.
    pub placed: Vec<PlacedSprite>,
    /// Actual atlas dimensions used.
    pub atlas_size: Size,
    /// Sprites that did not fit; forwarded to the next sheet in multipack mode.
    pub overflow: Vec<Sprite>,
}

/// Common interface implemented by all packing algorithms.
pub trait Packer: Send + Sync {
    /// Pack sprites onto a single sheet and return placements.
    fn pack(&self, input: PackInput) -> Result<PackOutput, CoreError>;

    /// Short human-readable name shown in log output.
    fn name(&self) -> &'static str;
}
