use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use fastpack_core::types::config::{PackMode, ScaleMode};

#[derive(Debug, Parser)]
#[command(name = "fastpack", about = "Texture atlas packer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

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
}

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
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Path to write the project file.
    #[arg(default_value = "project.fpsheet")]
    pub output: PathBuf,
}

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
    Fast,
    Good,
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
    Smooth,
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
