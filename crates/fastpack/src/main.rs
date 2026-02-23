mod cli;
mod error;
mod pipeline;
mod progress;
mod project;
mod split;
mod watch;

use anyhow::{Result, bail};
use clap::Parser;
use fastpack_core::types::config::Project;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Pack(args) => {
            let (inputs, output_dir, name, max_width, max_height, pack_mode) =
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
                    )
                };

            let result = pipeline::run_pack(pipeline::PackArgs {
                inputs,
                output_dir,
                name,
                max_width,
                max_height,
                pack_mode,
                detect_aliases: true,
            })?;

            let alias_note = if result.alias_count > 0 {
                format!(" ({} aliases)", result.alias_count)
            } else {
                String::new()
            };
            println!(
                "Packed {} sprites{} → {}×{} atlas → {} ({:.1} KB)",
                result.sprite_count,
                alias_note,
                result.atlas_size.w,
                result.atlas_size.h,
                result.texture_path.display(),
                result.texture_bytes as f64 / 1024.0,
            );
            println!(
                "Saved {} ({} bytes)",
                result.data_path.display(),
                result.data_bytes
            );

            if result.overflow_count > 0 {
                eprintln!(
                    "warning: {} sprite(s) did not fit and were dropped",
                    result.overflow_count
                );
            }

            Ok(())
        }

        cli::Commands::Init(args) => {
            let proj = Project::default();
            project::save(&proj, &args.output)?;
            println!("Wrote {}", args.output.display());
            Ok(())
        }
    }
}
