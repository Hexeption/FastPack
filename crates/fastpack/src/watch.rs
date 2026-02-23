use std::{path::PathBuf, sync::mpsc, time::Duration};

use anyhow::Result;
use fastpack_core::types::config::PackMode;
use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::notify::RecursiveMode;

use crate::pipeline::{PackArgs, run_pack};

/// Arguments for watch mode; mirrors `PackArgs`.
pub struct WatchArgs {
    pub inputs: Vec<PathBuf>,
    pub output_dir: PathBuf,
    pub name: String,
    pub max_width: u32,
    pub max_height: u32,
    pub pack_mode: PackMode,
    pub detect_aliases: bool,
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
    })?;

    let alias_note = if result.alias_count > 0 {
        format!(" ({} aliases)", result.alias_count)
    } else {
        String::new()
    };
    println!(
        "Packed {} sprites{} → {}×{} → {} ({:.1} KB)",
        result.sprite_count,
        alias_note,
        result.atlas_size.w,
        result.atlas_size.h,
        result.texture_path.display(),
        result.texture_bytes as f64 / 1024.0,
    );

    if result.overflow_count > 0 {
        eprintln!(
            "warning: {} sprite(s) did not fit and were dropped",
            result.overflow_count
        );
    }

    Ok(())
}
