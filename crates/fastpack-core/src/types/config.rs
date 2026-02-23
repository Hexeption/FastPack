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
    pub max_width: u32,
    pub max_height: u32,
    pub fixed_width: Option<u32>,
    pub fixed_height: Option<u32>,
    pub size_constraint: SizeConstraint,
    pub force_square: bool,
    pub allow_rotation: bool,
    pub pack_mode: PackMode,
    /// Transparent pixels added around the entire atlas edge.
    pub border_padding: u32,
    /// Transparent gap between every pair of adjacent sprites.
    pub shape_padding: u32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            max_width: 4096,
            max_height: 4096,
            fixed_width: None,
            fixed_height: None,
            size_constraint: SizeConstraint::AnySize,
            force_square: false,
            allow_rotation: true,
            pack_mode: PackMode::Good,
            border_padding: 2,
            shape_padding: 2,
        }
    }
}

/// Sprite pre-processing options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteConfig {
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
    pub detect_aliases: bool,
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

/// Output file format and path settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Base name for output files (no extension).
    pub name: String,
    /// Directory where output files are written.
    pub directory: PathBuf,
    pub texture_format: TextureFormat,
    pub pixel_format: PixelFormat,
    /// Multiply RGB by alpha before encoding.
    pub premultiply_alpha: bool,
    /// Data format key (e.g. `"json_hash"`, `"phaser3"`, `"pixijs"`).
    pub data_format: String,
    /// Quality for lossy codecs (0–100).
    pub quality: u8,
    /// Path prefix inserted into texture filenames within data files.
    pub texture_path_prefix: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            name: "atlas".to_string(),
            directory: PathBuf::from("output"),
            texture_format: TextureFormat::Png,
            pixel_format: PixelFormat::Rgba8888,
            premultiply_alpha: false,
            data_format: "json_hash".to_string(),
            quality: 95,
            texture_path_prefix: String::new(),
        }
    }
}

/// Algorithm-specific settings stored as a tagged enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlgorithmConfig {
    Grid { cell_width: u32, cell_height: u32 },
    Basic,
    MaxRects { heuristic: MaxRectsHeuristic },
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
    pub pivot: Option<Point>,
    pub nine_patch: Option<super::sprite::NinePatch>,
}

/// Top-level packer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackerConfig {
    pub layout: LayoutConfig,
    pub sprites: SpriteConfig,
    pub output: OutputConfig,
    pub algorithm: AlgorithmConfig,
    pub variants: Vec<ScaleVariant>,
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
    pub path: PathBuf,
    /// Glob pattern relative to `path`.
    pub filter: String,
}

impl Default for SourceSpec {
    fn default() -> Self {
        Self {
            path: PathBuf::from("sprites"),
            filter: "**/*.png".to_string(),
        }
    }
}

/// Full project as stored in a `.fpsheet` TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub config: PackerConfig,
    pub sources: Vec<SourceSpec>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            config: PackerConfig::default(),
            sources: vec![SourceSpec::default()],
        }
    }
}
