use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{
    pixel_format::{PixelFormat, TextureFormat},
    rect::Point,
};

/// How transparent borders are handled before packing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrimMode {
    /// Pack the full image including all transparent borders.
    None,
    /// Strip transparent borders; store the offset so engines can restore position.
    #[default]
    Trim,
    /// Crop tightly; the packed frame rect reflects only the opaque region.
    Crop,
    /// Like `Crop` but `SourceRect` offsets may be negative to keep original registration.
    CropKeepPos,
    /// Build a convex hull polygon; pack the hull's axis-aligned bounding box.
    Polygon,
}

impl std::fmt::Display for TrimMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Trim => write!(f, "trim"),
            Self::Crop => write!(f, "crop"),
            Self::CropKeepPos => write!(f, "crop-keep-pos"),
            Self::Polygon => write!(f, "polygon"),
        }
    }
}

impl std::str::FromStr for TrimMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "trim" => Ok(Self::Trim),
            "crop" => Ok(Self::Crop),
            "crop-keep-pos" | "cropkeeppos" => Ok(Self::CropKeepPos),
            "polygon" => Ok(Self::Polygon),
            _ => Err(format!("unknown trim mode: {s}")),
        }
    }
}

/// Constraint applied to the atlas texture dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SizeConstraint {
    /// No constraint — use the smallest rectangle that fits.
    #[default]
    AnySize,
    /// Width and height must both be powers of two.
    Pot,
    /// Width and height must each be divisible by 4.
    MultipleOf4,
    /// Width and height must each be divisible by 2.
    WordAligned,
}

impl std::str::FromStr for SizeConstraint {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "any" | "anysize" | "any-size" => Ok(Self::AnySize),
            "pot" => Ok(Self::Pot),
            "multipleof4" | "multiple-of-4" => Ok(Self::MultipleOf4),
            "wordaligned" | "word-aligned" => Ok(Self::WordAligned),
            _ => Err(format!("unknown size constraint: {s}")),
        }
    }
}

impl SizeConstraint {
    /// Round `size` up to satisfy this constraint.
    pub fn apply(self, size: u32) -> u32 {
        match self {
            Self::AnySize => size,
            Self::Pot => {
                if size <= 1 {
                    1
                } else if size.is_power_of_two() {
                    size
                } else {
                    size.next_power_of_two()
                }
            }
            Self::MultipleOf4 => {
                let r = size % 4;
                if r == 0 { size } else { size + (4 - r) }
            }
            Self::WordAligned => {
                let r = size % 2;
                if r == 0 { size } else { size + 1 }
            }
        }
    }
}

/// Speed vs. density trade-off for the packing search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackMode {
    /// Single-pass — fastest, largest atlas.
    Fast,
    /// Binary-search on width — balanced (recommended).
    #[default]
    Good,
    /// Exhaustive search — densest atlas, slowest.
    Best,
}

impl std::str::FromStr for PackMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fast" => Ok(Self::Fast),
            "good" => Ok(Self::Good),
            "best" => Ok(Self::Best),
            _ => Err(format!("unknown pack mode: {s}")),
        }
    }
}

/// Which packing algorithm to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlgorithmChoice {
    /// Equal-size grid; no data file required.
    Grid,
    /// Top-to-bottom strip fill — fast, no data file benefit.
    Basic,
    /// Rectangle bin-packing that minimises atlas area (recommended).
    #[default]
    MaxRects,
    /// Tight polygon-level packing (requires engine support).
    Polygon,
}

impl std::str::FromStr for AlgorithmChoice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "grid" => Ok(Self::Grid),
            "basic" => Ok(Self::Basic),
            "max-rects" | "maxrects" => Ok(Self::MaxRects),
            "polygon" => Ok(Self::Polygon),
            _ => Err(format!("unknown algorithm: {s}")),
        }
    }
}

/// Placement heuristic for the MaxRects algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MaxRectsHeuristic {
    /// Minimise the shorter leftover side after placement.
    #[default]
    BestShortSideFit,
    /// Minimise the longer leftover side after placement.
    BestLongSideFit,
    /// Minimise wasted area after placement.
    BestAreaFit,
    /// Topmost then leftmost placement.
    BottomLeftRule,
    /// Maximise contact perimeter with already-placed sprites and walls.
    ContactPointRule,
}

impl std::str::FromStr for MaxRectsHeuristic {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "best-short-side-fit" | "bssf" => Ok(Self::BestShortSideFit),
            "best-long-side-fit" | "blsf" => Ok(Self::BestLongSideFit),
            "best-area-fit" | "baf" => Ok(Self::BestAreaFit),
            "bottom-left-rule" | "blr" => Ok(Self::BottomLeftRule),
            "contact-point-rule" | "cpr" => Ok(Self::ContactPointRule),
            _ => Err(format!("unknown maxrects heuristic: {s}")),
        }
    }
}

/// Resampling algorithm used when producing scale variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScaleMode {
    /// Lanczos3 — high quality.
    #[default]
    Smooth,
    /// Nearest-neighbour — fast, crisp pixel art.
    Fast,
    /// EPX / Scale2x pixel art integer scaler.
    Scale2x,
    /// Scale3x — 3× EPX variant.
    Scale3x,
    /// HQ2x — look-up table anti-aliasing scaler.
    Hq2x,
    /// Eagle2x pixel art scaler.
    Eagle,
}

impl std::str::FromStr for ScaleMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "smooth" => Ok(Self::Smooth),
            "fast" => Ok(Self::Fast),
            "scale2x" => Ok(Self::Scale2x),
            "scale3x" => Ok(Self::Scale3x),
            "hq2x" => Ok(Self::Hq2x),
            "eagle" => Ok(Self::Eagle),
            _ => Err(format!("unknown scale mode: {s}")),
        }
    }
}

/// Atlas layout and dimension constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Maximum atlas width in pixels. Sprites wider than this cannot be packed.
    pub max_width: u32,
    /// Maximum atlas height in pixels. Sprites taller than this cannot be packed.
    pub max_height: u32,
    /// Pin the atlas width to an exact value, bypassing the size-fit algorithm.
    pub fixed_width: Option<u32>,
    /// Pin the atlas height to an exact value, bypassing the size-fit algorithm.
    pub fixed_height: Option<u32>,
    /// Rounding constraint applied to the computed atlas dimensions.
    pub size_constraint: SizeConstraint,
    /// When `true`, the atlas width and height are forced to be equal.
    pub force_square: bool,
    /// When `true`, sprites may be rotated 90° clockwise to improve packing density.
    pub allow_rotation: bool,
    /// Packing effort level (affects time but not algorithm choice).
    pub pack_mode: PackMode,
    /// Transparent pixels added around the entire atlas edge.
    pub border_padding: u32,
    /// Transparent gap between every pair of adjacent sprites.
    pub shape_padding: u32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            max_width: 2048,
            max_height: 2048,
            fixed_width: None,
            fixed_height: None,
            size_constraint: SizeConstraint::AnySize,
            force_square: false,
            allow_rotation: false,
            pack_mode: PackMode::Best,
            border_padding: 2,
            shape_padding: 2,
        }
    }
}

/// Sprite pre-processing options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteConfig {
    /// How transparent borders are stripped before packing.
    pub trim_mode: TrimMode,
    /// Alpha threshold: pixels at or below this level are treated as transparent.
    pub trim_threshold: u8,
    /// Pixels of transparent margin to preserve around trimmed edges.
    pub trim_margin: u32,
    /// Border pixel repetition count to prevent texture bleeding.
    pub extrude: u32,
    /// Width must be divisible by this value (0 = disabled).
    pub common_divisor_x: u32,
    /// Height must be divisible by this value (0 = disabled).
    pub common_divisor_y: u32,
    /// When `true`, duplicate sprites share a single atlas entry.
    pub detect_aliases: bool,
    /// Pivot applied to sprites that have no per-sprite override.
    pub default_pivot: Point,
}

impl Default for SpriteConfig {
    fn default() -> Self {
        Self {
            trim_mode: TrimMode::Trim,
            trim_threshold: 1,
            trim_margin: 0,
            extrude: 0,
            common_divisor_x: 0,
            common_divisor_y: 0,
            detect_aliases: true,
            default_pivot: Point::default(),
        }
    }
}

/// Output data serialization format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataFormat {
    /// Generic JSON object keyed by sprite name.
    JsonHash,
    /// JSON array of frame objects.
    JsonArray,
    /// Phaser 3 multi-atlas format.
    #[default]
    Phaser3,
    /// PixiJS sprite sheet format.
    Pixijs,
}

/// Output file format and path settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Base name for output files (no extension).
    pub name: String,
    /// Directory where output files are written.
    pub directory: PathBuf,
    /// Container format for the atlas texture.
    pub texture_format: TextureFormat,
    /// Pixel encoding within the texture.
    pub pixel_format: PixelFormat,
    /// Multiply RGB by alpha before encoding.
    pub premultiply_alpha: bool,
    /// Output data serialization format.
    pub data_format: DataFormat,
    /// Quality for lossy codecs (0–100).
    pub quality: u8,
    /// Path prefix inserted into texture filenames within data files.
    pub texture_path_prefix: String,
    /// When `true`, overflow sprites are packed into additional sheets.
    pub multipack: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            name: "atlas".to_string(),
            directory: PathBuf::from("output"),
            texture_format: TextureFormat::Png,
            pixel_format: PixelFormat::Rgba8888,
            premultiply_alpha: false,
            data_format: DataFormat::Phaser3,
            quality: 95,
            texture_path_prefix: String::new(),
            multipack: false,
        }
    }
}

/// Algorithm-specific settings stored as a tagged enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlgorithmConfig {
    /// Equal-size grid placement. `cell_width` and `cell_height` set cell dimensions (0 = auto).
    Grid {
        /// Cell width in pixels. 0 sizes to the widest sprite in the set.
        cell_width: u32,
        /// Cell height in pixels. 0 sizes to the tallest sprite in the set.
        cell_height: u32,
    },
    /// Top-to-bottom row-strip placement (fast, no width search).
    Basic,
    /// MaxRects bin-packing with a configurable placement heuristic.
    MaxRects {
        /// Heuristic used to score candidate free-rect placements.
        heuristic: MaxRectsHeuristic,
    },
    /// Tight polygon-level packing (requires engine support for hull meshes).
    Polygon,
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self::MaxRects {
            heuristic: MaxRectsHeuristic::default(),
        }
    }
}

/// One scale variant (e.g. @1x, @2x).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleVariant {
    /// Scale factor applied to every sprite (1.0 = full resolution).
    pub scale: f32,
    /// Suffix appended to output filenames (e.g. `"@1x"`).
    pub suffix: String,
    /// Resampling filter used when scaling sprites.
    pub scale_mode: ScaleMode,
}

impl Default for ScaleVariant {
    fn default() -> Self {
        Self {
            scale: 1.0,
            suffix: String::new(),
            scale_mode: ScaleMode::Smooth,
        }
    }
}

/// Per-sprite metadata override from the `.fpsheet` project file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteOverride {
    /// Sprite id (relative path without extension).
    pub id: String,
    /// Pivot override. `None` inherits from `SpriteConfig::default_pivot`.
    pub pivot: Option<Point>,
    /// 9-patch border override. `None` means the sprite is not a 9-patch.
    pub nine_patch: Option<super::sprite::NinePatch>,
}

/// Top-level packer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackerConfig {
    /// Atlas layout and size constraints.
    pub layout: LayoutConfig,
    /// Sprite pre-processing settings.
    pub sprites: SpriteConfig,
    /// Output format and path settings.
    pub output: OutputConfig,
    /// Algorithm and its settings.
    pub algorithm: AlgorithmConfig,
    /// Scale variants to produce (usually one entry for the primary scale).
    pub variants: Vec<ScaleVariant>,
    /// Per-sprite overrides applied after loading.
    pub sprite_overrides: Vec<SpriteOverride>,
}

impl Default for PackerConfig {
    fn default() -> Self {
        Self {
            layout: LayoutConfig::default(),
            sprites: SpriteConfig::default(),
            output: OutputConfig::default(),
            algorithm: AlgorithmConfig::default(),
            variants: vec![ScaleVariant::default()],
            sprite_overrides: Vec::new(),
        }
    }
}

/// Glob-based source input specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpec {
    /// Root directory to search.
    pub path: PathBuf,
    /// Glob pattern relative to `path`.
    pub filter: String,
}

/// Full project as stored in a `.fpsheet` TOML file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Project {
    /// Packer configuration for this project.
    #[serde(flatten)]
    pub config: PackerConfig,
    /// Input directories and glob filters.
    pub sources: Vec<SourceSpec>,
}
