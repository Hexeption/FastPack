mod cli;
mod error;
mod pipeline;
mod progress;
mod project;
mod split;
mod watch;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Pack(args) => {
            let result = pipeline::run_pack(pipeline::PackArgs {
                inputs: args.inputs,
                output_dir: args.output,
                name: args.name,
                max_width: args.max_width,
                max_height: args.max_height,
                pack_mode: args.pack_mode.into(),
            })?;

            println!(
                "Packed {} sprites → {}×{} atlas → {} ({:.1} KB)",
                result.sprite_count,
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
    }
}
