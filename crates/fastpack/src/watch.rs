use std::{path::PathBuf, sync::mpsc, time::Duration};

use anyhow::Result;
use fastpack_core::types::{
    config::{PackMode, ScaleVariant, SpriteOverride},
    rect::Point,
};
use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::notify::RecursiveMode;

use crate::pipeline::{PackArgs, run_pack};

/// Arguments for watch mode; mirrors `PackArgs`.
pub struct WatchArgs {
    /// Input directories or individual image files to watch.
    pub inputs: Vec<PathBuf>,
    /// Directory where atlas texture and data files are written.
    pub output_dir: PathBuf,
    /// Base name for output files (no extension).
    pub name: String,
    /// Maximum atlas width in pixels.
    pub max_width: u32,
    /// Maximum atlas height in pixels.
    pub max_height: u32,
    /// Packing effort level; controls speed vs. atlas density trade-off.
    pub pack_mode: PackMode,
    /// When `true`, pixel-identical sprites share a single atlas frame.
    pub detect_aliases: bool,
    /// Emit additional sheets when sprites overflow the first atlas.
    pub multipack: bool,
    /// Pivot applied to every frame that has no per-sprite override.
    pub default_pivot: Option<Point>,
    /// Per-sprite metadata (pivot, nine-patch) read from the project file.
    pub sprite_overrides: Vec<SpriteOverride>,
    /// Scale variants to produce. An empty list is treated as a single @1x variant.
    pub variants: Vec<ScaleVariant>,
    /// Export data format identifier (e.g. `"json_hash"`, `"phaser3"`).
    pub data_format: String,
}

/// Watch input directories and repack on any change.
///
/// Runs an initial pack immediately, then watches all input paths for
/// filesystem events. Repacks after a 500 ms debounce each time a change
/// is detected.
pub fn run_watch(args: WatchArgs) -> Result<()> {
    println!(
        "Watching {} path(s). Press Ctrl-C to stop.",
        args.inputs.len()
    );
    run_once(&args)?;

    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(500), tx)?;

    for input in &args.inputs {
        let watch_path = if input.is_file() {
            input.parent().unwrap_or(input.as_path())
        } else {
            input.as_path()
        };
        debouncer
            .watcher()
            .watch(watch_path, RecursiveMode::Recursive)?;
    }

    loop {
        match rx.recv() {
            Ok(Ok(_)) => {
                if let Err(e) = run_once(&args) {
                    eprintln!("error: {e}");
                }
            }
            Ok(Err(errs)) => eprintln!("watch error: {errs:?}"),
            Err(_) => break,
        }
    }

    Ok(())
}

fn run_once(args: &WatchArgs) -> Result<()> {
    let result = run_pack(PackArgs {
        inputs: args.inputs.clone(),
        output_dir: args.output_dir.clone(),
        name: args.name.clone(),
        max_width: args.max_width,
        max_height: args.max_height,
        pack_mode: args.pack_mode,
        detect_aliases: args.detect_aliases,
        multipack: args.multipack,
        default_pivot: args.default_pivot,
        sprite_overrides: args.sprite_overrides.clone(),
        variants: args.variants.clone(),
        data_format: args.data_format.clone(),
    })?;

    let alias_note = if result.alias_count > 0 {
        format!(" ({} aliases)", result.alias_count)
    } else {
        String::new()
    };

    for sheet in &result.sheets {
        println!(
            "Packed {} sprites{} → {}×{} → {} ({:.1} KB)",
            result.sprite_count,
            alias_note,
            sheet.atlas_size.w,
            sheet.atlas_size.h,
            sheet.texture_path.display(),
            sheet.texture_bytes as f64 / 1024.0,
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
