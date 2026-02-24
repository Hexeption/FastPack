use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use fastpack_core::types::{
    config::{DataFormat, PackMode, ScaleMode, SizeConstraint, TrimMode},
    pixel_format::{PixelFormat, TextureFormat},
};

/// Root CLI entry point parsed by clap.
#[derive(Debug, Parser)]
#[command(name = "fastpack", about = "Texture atlas packer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Top-level subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Pack sprites from one or more directories or files into a texture atlas.
    Pack(PackArgs),
    /// Watch input directories and repack on any change.
    Watch(PackArgs),
    /// Write a default .fpsheet project file to disk.
    Init(InitArgs),
    /// Split a packed atlas back into individual sprite files.
    Split(SplitArgs),
    /// Open the graphical user interface (default when no subcommand is given).
    Gui(GuiArgs),
}

/// Arguments for the `gui` subcommand.
#[derive(Debug, Args)]
pub struct GuiArgs {
    /// Optional .fpsheet project file to open on startup.
    #[arg(value_name = "PROJECT")]
    pub project: Option<PathBuf>,
}

/// Arguments shared by the `pack` and `watch` subcommands.
#[derive(Debug, Args)]
pub struct PackArgs {
    /// Input directories or files to search for sprites.
    #[arg(value_name = "INPUT")]
    pub inputs: Vec<PathBuf>,

    /// Load settings from a .fpsheet project file.
    #[arg(long, value_name = "FILE")]
    pub project: Option<PathBuf>,

    /// Output directory for the generated atlas and data file.
    #[arg(short, long, default_value = "output")]
    pub output: PathBuf,

    /// Base name for output files (without extension).
    #[arg(long, default_value = "atlas")]
    pub name: String,

    /// Maximum atlas width in pixels.
    #[arg(long, default_value_t = 4096)]
    pub max_width: u32,

    /// Maximum atlas height in pixels.
    #[arg(long, default_value_t = 4096)]
    pub max_height: u32,

    /// Compression effort level.
    #[arg(long, value_enum, default_value = "good")]
    pub pack_mode: PackModeArg,

    /// Constraint applied to atlas dimensions (any, pot, multiple-of-4, word-aligned).
    #[arg(long, value_enum, default_value = "any")]
    pub size_constraint: SizeConstraintArg,

    /// Force the atlas to be square (width == height).
    #[arg(long)]
    pub force_square: bool,

    /// Allow 90° sprite rotation to improve packing density.
    #[arg(long, default_value_t = true)]
    pub allow_rotation: bool,

    /// Transparent pixels added around the atlas edge.
    #[arg(long, default_value_t = 2)]
    pub border_padding: u32,

    /// Transparent gap between adjacent sprites.
    #[arg(long, default_value_t = 2)]
    pub shape_padding: u32,

    /// How to strip transparent borders from sprites.
    #[arg(long, value_enum, default_value = "trim")]
    pub trim_mode: TrimModeArg,

    /// Pixels of transparent margin to keep around trimmed edges.
    #[arg(long, default_value_t = 0)]
    pub trim_margin: u32,

    /// Alpha threshold: pixels at or below this value are considered transparent.
    #[arg(long, default_value_t = 1)]
    pub trim_threshold: u8,

    /// Pixels of border extrusion added to each sprite edge.
    #[arg(long, default_value_t = 0)]
    pub extrude: u32,

    /// Deduplicate pixel-identical sprites as aliases.
    #[arg(long, default_value_t = true)]
    pub detect_aliases: bool,

    /// Emit additional sheets when sprites overflow the first atlas.
    #[arg(long)]
    pub multipack: bool,

    /// Default pivot X coordinate (0.0–1.0). Requires --pivot-y.
    #[arg(long, value_name = "X")]
    pub pivot_x: Option<f32>,

    /// Default pivot Y coordinate (0.0–1.0). Requires --pivot-x.
    #[arg(long, value_name = "Y")]
    pub pivot_y: Option<f32>,

    /// Scale factor applied to output (e.g. 0.5 produces a half-resolution atlas).
    #[arg(long, default_value_t = 1.0)]
    pub scale: f32,

    /// Suffix appended to output filenames (e.g. @2x produces atlas@2x.png).
    #[arg(long, default_value = "")]
    pub suffix: String,

    /// Resampling filter used when scaling.
    #[arg(long, value_enum, default_value = "smooth")]
    pub scale_mode: ScaleModeArg,

    /// Output data format.
    #[arg(long, value_enum, default_value = "json-hash")]
    pub data_format: DataFormatArg,

    /// Output texture format (png, jpeg, webp, dxt1, dxt5).
    #[arg(long, value_enum, default_value = "png")]
    pub texture_format: TextureFormatArg,

    /// Pixel bit depth; Floyd-Steinberg dithering is applied when not rgba8888.
    #[arg(long, value_enum, default_value = "rgba8888")]
    pub pixel_format: PixelFormatArg,
}

/// Arguments for the `init` subcommand.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Path to write the project file.
    #[arg(default_value = "project.fpsheet")]
    pub output: PathBuf,
}

/// Arguments for the `split` subcommand.
#[derive(Debug, Args)]
pub struct SplitArgs {
    /// Path to the packed atlas PNG.
    #[arg(value_name = "ATLAS")]
    pub atlas: PathBuf,

    /// Path to the JSON Hash data file produced by fastpack.
    #[arg(value_name = "DATA")]
    pub data: PathBuf,

    /// Output directory for extracted sprite files.
    #[arg(short, long, default_value = "sprites")]
    pub output_dir: PathBuf,
}

/// Clap-facing pack mode enum; converts to `fastpack_core::types::config::PackMode`.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum PackModeArg {
    /// Single-pass basic strip packer; fastest, largest atlas.
    Fast,
    /// MaxRects single-pass; good density, moderate speed. Default.
    Good,
    /// MaxRects width search; densest atlas, slowest.
    Best,
}

impl From<PackModeArg> for PackMode {
    fn from(arg: PackModeArg) -> Self {
        match arg {
            PackModeArg::Fast => PackMode::Fast,
            PackModeArg::Good => PackMode::Good,
            PackModeArg::Best => PackMode::Best,
        }
    }
}

/// Clap-facing scale mode enum; converts to `fastpack_core::types::config::ScaleMode`.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ScaleModeArg {
    /// Lanczos3 resampling; high quality. Default.
    Smooth,
    /// Nearest-neighbour; crisp pixel art, no blurring.
    Fast,
}

impl From<ScaleModeArg> for ScaleMode {
    fn from(arg: ScaleModeArg) -> Self {
        match arg {
            ScaleModeArg::Smooth => ScaleMode::Smooth,
            ScaleModeArg::Fast => ScaleMode::Fast,
        }
    }
}

/// Clap-facing size constraint enum.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SizeConstraintArg {
    /// No constraint — smallest rectangle that fits.
    Any,
    /// Width and height must be powers of two.
    Pot,
    /// Width and height must each be divisible by 4.
    MultipleOf4,
    /// Width and height must each be divisible by 2.
    WordAligned,
}

impl From<SizeConstraintArg> for SizeConstraint {
    fn from(arg: SizeConstraintArg) -> Self {
        match arg {
            SizeConstraintArg::Any => SizeConstraint::AnySize,
            SizeConstraintArg::Pot => SizeConstraint::Pot,
            SizeConstraintArg::MultipleOf4 => SizeConstraint::MultipleOf4,
            SizeConstraintArg::WordAligned => SizeConstraint::WordAligned,
        }
    }
}

/// Clap-facing trim mode enum.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum TrimModeArg {
    /// Pack the full image including transparent borders.
    None,
    /// Strip transparent borders; store offset for engine reconstruction.
    Trim,
    /// Crop tightly to the opaque region.
    Crop,
    /// Like Crop but offsets may be negative to keep original registration.
    CropKeepPos,
    /// Build convex hull polygon; pack its bounding box.
    Polygon,
}

impl From<TrimModeArg> for TrimMode {
    fn from(arg: TrimModeArg) -> Self {
        match arg {
            TrimModeArg::None => TrimMode::None,
            TrimModeArg::Trim => TrimMode::Trim,
            TrimModeArg::Crop => TrimMode::Crop,
            TrimModeArg::CropKeepPos => TrimMode::CropKeepPos,
            TrimModeArg::Polygon => TrimMode::Polygon,
        }
    }
}

/// Clap-facing data format enum.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum DataFormatArg {
    /// Generic JSON object keyed by sprite name.
    JsonHash,
    /// JSON array of frame objects.
    JsonArray,
    /// Phaser 3 multi-atlas format.
    Phaser3,
    /// PixiJS sprite sheet format.
    Pixijs,
}

impl From<DataFormatArg> for DataFormat {
    fn from(arg: DataFormatArg) -> Self {
        match arg {
            DataFormatArg::JsonHash => DataFormat::JsonHash,
            DataFormatArg::JsonArray => DataFormat::JsonArray,
            DataFormatArg::Phaser3 => DataFormat::Phaser3,
            DataFormatArg::Pixijs => DataFormat::Pixijs,
        }
    }
}

/// Clap-facing texture format enum.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum TextureFormatArg {
    /// Lossless PNG. Default.
    Png,
    /// Lossy JPEG; no alpha channel.
    Jpeg,
    /// WebP (lossless or lossy depending on pack mode).
    Webp,
    /// DXT1 / BC1 hardware compression; no per-pixel alpha.
    Dxt1,
    /// DXT5 / BC3 hardware compression; full alpha channel.
    Dxt5,
}

impl From<TextureFormatArg> for TextureFormat {
    fn from(arg: TextureFormatArg) -> Self {
        match arg {
            TextureFormatArg::Png => TextureFormat::Png,
            TextureFormatArg::Jpeg => TextureFormat::Jpeg,
            TextureFormatArg::Webp => TextureFormat::WebP,
            TextureFormatArg::Dxt1 => TextureFormat::Dxt1,
            TextureFormatArg::Dxt5 => TextureFormat::Dxt5,
        }
    }
}

/// Clap-facing pixel format enum.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum PixelFormatArg {
    /// 32-bit RGBA (8 bits per channel). Default; no dithering applied.
    Rgba8888,
    /// 24-bit RGB (8 bits per channel, alpha forced to 255).
    Rgb888,
    /// 16-bit RGB (5-6-5). Floyd-Steinberg dithering applied.
    Rgb565,
    /// 16-bit RGBA (4 bits per channel). Floyd-Steinberg dithering applied.
    Rgba4444,
    /// 16-bit RGBA (5-5-5-1). Floyd-Steinberg dithering; alpha thresholded at 128.
    Rgba5551,
    /// 8-bit alpha only.
    Alpha8,
}

impl From<PixelFormatArg> for PixelFormat {
    fn from(arg: PixelFormatArg) -> Self {
        match arg {
            PixelFormatArg::Rgba8888 => PixelFormat::Rgba8888,
            PixelFormatArg::Rgb888 => PixelFormat::Rgb888,
            PixelFormatArg::Rgb565 => PixelFormat::Rgb565,
            PixelFormatArg::Rgba4444 => PixelFormat::Rgba4444,
            PixelFormatArg::Rgba5551 => PixelFormat::Rgba5551,
            PixelFormatArg::Alpha8 => PixelFormat::Alpha8,
        }
    }
}
