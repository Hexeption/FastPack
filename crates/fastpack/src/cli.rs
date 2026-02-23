use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use fastpack_core::types::config::PackMode;

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
    /// Write a default .fpsheet project file to disk.
    Init(InitArgs),
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
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Path to write the project file.
    #[arg(default_value = "project.fpsheet")]
    pub output: PathBuf,
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
