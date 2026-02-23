use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImage, GenericImageView};
use serde::Deserialize;

pub struct SplitArgs {
    pub atlas_path: PathBuf,
    pub data_path: PathBuf,
    pub output_dir: PathBuf,
}

pub struct SplitResult {
    pub sprite_count: usize,
    pub output_dir: PathBuf,
}

/// Split a packed atlas texture back into individual sprite files.
///
/// Reads the atlas PNG and the JSON Hash data file. For each frame, crops the
/// atlas region, restores the original canvas size if the sprite was trimmed,
/// and writes the result to `<output_dir>/<id>.png`. Intermediate directories
/// are created as needed.
pub fn run_split(args: SplitArgs) -> Result<SplitResult> {
    let atlas = image::open(&args.atlas_path)
        .with_context(|| format!("failed to open atlas {}", args.atlas_path.display()))?;

    let data = std::fs::read_to_string(&args.data_path)
        .with_context(|| format!("failed to read data file {}", args.data_path.display()))?;
    let doc: JsonHashDoc =
        serde_json::from_str(&data).context("failed to parse atlas data file")?;

    std::fs::create_dir_all(&args.output_dir).context("failed to create output directory")?;

    let mut sprite_count = 0;
    for (id, frame_data) in &doc.frames {
        let f = &frame_data.frame;
        let sss = &frame_data.sprite_source_size;
        let ss = &frame_data.source_size;

        let cropped = atlas.view(f.x, f.y, f.w, f.h).to_image();
        let cropped = DynamicImage::ImageRgba8(cropped);

        let sprite = if frame_data.rotated {
            cropped.rotate270()
        } else {
            cropped
        };

        let out_image = if frame_data.trimmed {
            let mut canvas = DynamicImage::new_rgba8(ss.w, ss.h);
            canvas
                .copy_from(&sprite, sss.x.max(0) as u32, sss.y.max(0) as u32)
                .context("failed to blit trimmed sprite")?;
            canvas
        } else {
            sprite
        };

        let out_path = output_path_for_id(id, &args.output_dir);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create dir for {}", out_path.display()))?;
        }
        out_image
            .save(&out_path)
            .with_context(|| format!("failed to save {}", out_path.display()))?;

        sprite_count += 1;
    }

    Ok(SplitResult {
        sprite_count,
        output_dir: args.output_dir,
    })
}

fn output_path_for_id(id: &str, base: &Path) -> PathBuf {
    let normalized = id.replace('/', std::path::MAIN_SEPARATOR_STR);
    base.join(format!("{normalized}.png"))
}

#[derive(Deserialize)]
struct JsonHashDoc {
    frames: HashMap<String, FrameEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrameEntry {
    frame: Rect,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: SourceRect,
    source_size: Size,
}

#[derive(Deserialize)]
struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Deserialize)]
struct SourceRect {
    x: i32,
    y: i32,
    #[allow(dead_code)]
    w: u32,
    #[allow(dead_code)]
    h: u32,
}

#[derive(Deserialize)]
struct Size {
    w: u32,
    h: u32,
}
