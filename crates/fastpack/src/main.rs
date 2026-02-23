mod cli;
mod error;
mod pipeline;
mod progress;
mod project;
mod split;
mod watch;

use anyhow::{Result, bail};
use clap::Parser;
use fastpack_core::types::{config::Project, rect::Point};

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Pack(args) => {
            let (inputs, output_dir, name, max_width, max_height, pack_mode, sprite_overrides) =
                if let Some(proj_path) = &args.project {
                    let proj = project::load(proj_path)?;
                    let inputs = if args.inputs.is_empty() {
                        proj.sources.iter().map(|s| s.path.clone()).collect()
                    } else {
                        args.inputs.clone()
                    };
                    (
                        inputs,
                        args.output.clone(),
                        proj.config.output.name.clone(),
                        proj.config.layout.max_width,
                        proj.config.layout.max_height,
                        proj.config.layout.pack_mode,
                        proj.config.sprite_overrides.clone(),
                    )
                } else {
                    if args.inputs.is_empty() {
                        bail!("no inputs specified; provide input paths or --project <file>");
                    }
                    (
                        args.inputs.clone(),
                        args.output.clone(),
                        args.name.clone(),
                        args.max_width,
                        args.max_height,
                        args.pack_mode.into(),
                        Vec::new(),
                    )
                };

            let default_pivot = match (args.pivot_x, args.pivot_y) {
                (Some(x), Some(y)) => Some(Point { x, y }),
                _ => None,
            };

            let result = pipeline::run_pack(pipeline::PackArgs {
                inputs,
                output_dir,
                name,
                max_width,
                max_height,
                pack_mode,
                detect_aliases: true,
                multipack: args.multipack,
                default_pivot,
                sprite_overrides,
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
                println!(
                    "Saved {} ({} bytes)",
                    sheet.data_path.display(),
                    sheet.data_bytes
                );
            }

            if result.overflow_count > 0 {
                eprintln!(
                    "warning: {} sprite(s) did not fit and were dropped",
                    result.overflow_count
                );
            }

            Ok(())
        }

        cli::Commands::Watch(args) => {
            let (inputs, output_dir, name, max_width, max_height, pack_mode, sprite_overrides) =
                if let Some(proj_path) = &args.project {
                    let proj = project::load(proj_path)?;
                    let inputs = if args.inputs.is_empty() {
                        proj.sources.iter().map(|s| s.path.clone()).collect()
                    } else {
                        args.inputs.clone()
                    };
                    (
                        inputs,
                        args.output.clone(),
                        proj.config.output.name.clone(),
                        proj.config.layout.max_width,
                        proj.config.layout.max_height,
                        proj.config.layout.pack_mode,
                        proj.config.sprite_overrides.clone(),
                    )
                } else {
                    if args.inputs.is_empty() {
                        bail!("no inputs specified; provide input paths or --project <file>");
                    }
                    (
                        args.inputs.clone(),
                        args.output.clone(),
                        args.name.clone(),
                        args.max_width,
                        args.max_height,
                        args.pack_mode.into(),
                        Vec::new(),
                    )
                };

            let default_pivot = match (args.pivot_x, args.pivot_y) {
                (Some(x), Some(y)) => Some(Point { x, y }),
                _ => None,
            };

            watch::run_watch(watch::WatchArgs {
                inputs,
                output_dir,
                name,
                max_width,
                max_height,
                pack_mode,
                detect_aliases: true,
                multipack: args.multipack,
                default_pivot,
                sprite_overrides,
            })?;
            Ok(())
        }

        cli::Commands::Init(args) => {
            let proj = Project::default();
            project::save(&proj, &args.output)?;
            println!("Wrote {}", args.output.display());
            Ok(())
        }

        cli::Commands::Split(args) => {
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
