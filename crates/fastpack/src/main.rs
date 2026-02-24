//! FastPack CLI: pack sprites into atlases, watch for changes, and export data files.
#![cfg_attr(windows, windows_subsystem = "windows")]
mod cli;
mod error;
mod pipeline;
mod progress;
mod project;
mod split;
mod watch;

use anyhow::{Result, bail};
use clap::Parser;
use fastpack_core::types::{
    config::{DataFormat, LayoutConfig, Project, ScaleVariant, SpriteConfig},
    pixel_format::{PixelFormat, TextureFormat},
    rect::Point,
};

fn main() -> Result<()> {
    // Re-attach to the parent console so CLI subcommands produce visible
    // output when launched from a terminal even though this is a GUI subsystem binary.
    #[cfg(windows)]
    unsafe {
        unsafe extern "system" {
            fn AttachConsole(dwProcessId: u32) -> i32;
        }
        AttachConsole(0xFFFF_FFFF);
    }

    let cli = cli::Cli::parse();
    match cli.command {
        None | Some(cli::Commands::Gui(cli::GuiArgs { project: None })) => fastpack_gui::run(None),

        Some(cli::Commands::Gui(cli::GuiArgs { project })) => fastpack_gui::run(project),

        Some(cli::Commands::Pack(args)) => {
            let (
                inputs,
                output_dir,
                name,
                layout,
                sprite_config,
                sprite_overrides,
                variants,
                data_format,
                texture_format,
                pixel_format,
            ) = resolve_pack_fields(&args)?;

            let default_pivot = match (args.pivot_x, args.pivot_y) {
                (Some(x), Some(y)) => Some(Point { x, y }),
                _ => None,
            };

            let result = pipeline::run_pack(pipeline::PackArgs {
                inputs,
                output_dir,
                name,
                layout,
                sprite_config,
                multipack: args.multipack,
                default_pivot,
                sprite_overrides,
                variants,
                data_format,
                texture_format,
                pixel_format,
            })?;

            let alias_note = if result.alias_count > 0 {
                format!(" ({} aliases)", result.alias_count)
            } else {
                String::new()
            };
            for sheet in &result.sheets {
                println!(
                    "Packed {} sprites{} → {}×{} atlas → {} ({:.1} KB)",
                    result.sprite_count,
                    alias_note,
                    sheet.atlas_size.w,
                    sheet.atlas_size.h,
                    sheet.texture_path.display(),
                    sheet.texture_bytes as f64 / 1024.0,
                );
                if sheet.data_bytes > 0 {
                    println!(
                        "Saved {} ({} bytes)",
                        sheet.data_path.display(),
                        sheet.data_bytes
                    );
                }
            }

            if result.overflow_count > 0 {
                eprintln!(
                    "warning: {} sprite(s) did not fit and were dropped",
                    result.overflow_count
                );
            }

            Ok(())
        }

        Some(cli::Commands::Watch(args)) => {
            let (
                inputs,
                output_dir,
                name,
                layout,
                sprite_config,
                sprite_overrides,
                variants,
                data_format,
                texture_format,
                pixel_format,
            ) = resolve_pack_fields(&args)?;

            let default_pivot = match (args.pivot_x, args.pivot_y) {
                (Some(x), Some(y)) => Some(Point { x, y }),
                _ => None,
            };

            watch::run_watch(watch::WatchArgs {
                inputs,
                output_dir,
                name,
                layout,
                sprite_config,
                multipack: args.multipack,
                default_pivot,
                sprite_overrides,
                variants,
                data_format,
                texture_format,
                pixel_format,
            })?;
            Ok(())
        }

        Some(cli::Commands::Init(args)) => {
            let proj = Project::default();
            project::save(&proj, &args.output)?;
            println!("Wrote {}", args.output.display());
            Ok(())
        }

        Some(cli::Commands::Split(args)) => {
            let result = split::run_split(split::SplitArgs {
                atlas_path: args.atlas,
                data_path: args.data,
                output_dir: args.output_dir,
            })?;
            println!(
                "Split {} sprite(s) → {}",
                result.sprite_count,
                result.output_dir.display()
            );
            Ok(())
        }
    }
}

type PackFields = (
    Vec<std::path::PathBuf>,
    std::path::PathBuf,
    String,
    LayoutConfig,
    SpriteConfig,
    Vec<fastpack_core::types::config::SpriteOverride>,
    Vec<ScaleVariant>,
    DataFormat,
    TextureFormat,
    PixelFormat,
);

/// Resolve pack fields from either a project file or bare CLI flags.
///
/// When `--project` is given, the project file provides all layout/sprite
/// settings; CLI inputs and output path still override the project defaults.
/// When no project is given, every setting comes directly from CLI flags.
fn resolve_pack_fields(args: &cli::PackArgs) -> Result<PackFields> {
    if let Some(proj_path) = &args.project {
        let proj = project::load(proj_path)?;
        let inputs = if args.inputs.is_empty() {
            proj.sources.iter().map(|s| s.path.clone()).collect()
        } else {
            args.inputs.clone()
        };
        Ok((
            inputs,
            args.output.clone(),
            proj.config.output.name.clone(),
            proj.config.layout.clone(),
            proj.config.sprites.clone(),
            proj.config.sprite_overrides.clone(),
            proj.config.variants.clone(),
            proj.config.output.data_format,
            args.texture_format.clone().into(),
            args.pixel_format.clone().into(),
        ))
    } else {
        if args.inputs.is_empty() {
            bail!("no inputs specified; provide input paths or --project <file>");
        }
        let layout = LayoutConfig {
            max_width: args.max_width,
            max_height: args.max_height,
            fixed_width: None,
            fixed_height: None,
            size_constraint: args.size_constraint.clone().into(),
            force_square: args.force_square,
            allow_rotation: args.allow_rotation,
            pack_mode: args.pack_mode.clone().into(),
            border_padding: args.border_padding,
            shape_padding: args.shape_padding,
        };
        let sprite_config = SpriteConfig {
            trim_mode: args.trim_mode.clone().into(),
            trim_threshold: args.trim_threshold,
            trim_margin: args.trim_margin,
            extrude: args.extrude,
            common_divisor_x: 0,
            common_divisor_y: 0,
            detect_aliases: args.detect_aliases,
            default_pivot: Point::default(),
        };
        let variant = ScaleVariant {
            scale: args.scale,
            suffix: args.suffix.clone(),
            scale_mode: args.scale_mode.clone().into(),
        };
        Ok((
            args.inputs.clone(),
            args.output.clone(),
            args.name.clone(),
            layout,
            sprite_config,
            Vec::new(),
            vec![variant],
            args.data_format.clone().into(),
            args.texture_format.clone().into(),
            args.pixel_format.clone().into(),
        ))
    }
}
