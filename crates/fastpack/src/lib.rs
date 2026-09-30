//! FastPack CLI: pack sprites into atlases, watch for changes, and export data files.
//!
//! Exposed as a library so the GUI binary can dispatch to the CLI when it is
//! launched with a subcommand (e.g. `FastPack.AppImage pack sprites/`).
mod cli;
mod error;
mod pipeline;
mod progress;
mod project;
mod split;
mod watch;

use anyhow::{Ok, Result, bail};
use clap::{CommandFactory, Parser};
use fastpack_core::types::{
    config::{DataFormat, LayoutConfig, Project, ScaleVariant, SpriteConfig},
    pixel_format::{PixelFormat, TextureFormat},
    rect::Point,
};

/// Returns true when `args` (excluding argv\[0\]) should be handled by the CLI
/// rather than opening the GUI: a known subcommand or a help/version flag.
pub fn is_cli_invocation<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let Some(first) = args.into_iter().next() else {
        return false;
    };
    let Some(first) = first.as_ref().to_str() else {
        return false;
    };
    matches!(first, "-h" | "--help" | "-V" | "--version" | "help")
        || cli::Cli::command()
            .get_subcommands()
            .any(|c| c.get_name() == first)
}

/// Parse `std::env::args` and run the requested CLI command.
pub fn run() -> Result<()> {
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
        None => {
            // No subcommand: show help
            cli::Cli::command().print_help()?;
            Ok(())
        }

        Some(cli::Commands::Pack(args)) => {
            let (
                inputs,
                output_dir,
                name,
                layout,
                sprite_config,
                sprite_overrides,
                excludes,
                variants,
                data_format,
                texture_format,
                pixel_format,
                premultiply_alpha,
                hide_name,
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
                premultiply_alpha,
                excludes,
                hide_name,
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
                excludes,
                variants,
                data_format,
                texture_format,
                pixel_format,
                premultiply_alpha,
                hide_name,
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
                premultiply_alpha,
                excludes,
                hide_name,
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
    Vec<String>,
    Vec<ScaleVariant>,
    DataFormat,
    TextureFormat,
    PixelFormat,
    bool, // premultiply_alpha
    bool, // hide_name
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
            {
                let mut sprites = proj.config.sprites.clone();
                sprites.alpha_bleed |= args.alpha_bleed;
                sprites
            },
            proj.config.sprite_overrides.clone(),
            proj.config.excludes.clone(),
            proj.config.variants.clone(),
            proj.config.output.data_format,
            args.texture_format.clone().into(),
            args.pixel_format.clone().into(),
            args.premultiply_alpha,
            proj.config.output.hide_name || args.hide_name,
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
            allow_rotation: !args.no_allow_rotation,
            pack_mode: args.pack_mode.clone().into(),
            border_padding: args.border_padding,
            shape_padding: args.shape_padding,
        };
        let sprite_config = SpriteConfig {
            trim_mode: args.trim_mode.clone().into(),
            trim_threshold: args.trim_threshold,
            trim_margin: args.trim_margin,
            extrude: args.extrude,
            alpha_bleed: args.alpha_bleed,
            common_divisor_x: 0,
            common_divisor_y: 0,
            detect_aliases: !args.no_detect_aliases,
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
            Vec::new(),
            vec![variant],
            args.data_format.clone().into(),
            args.texture_format.clone().into(),
            args.pixel_format.clone().into(),
            args.premultiply_alpha,
            args.hide_name,
        ))
    }
}
