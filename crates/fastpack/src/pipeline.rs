use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use fastpack_compress::{
    backends::png::PngCompressor,
    compressor::{CompressInput, Compressor},
};
use fastpack_core::{
    algorithms::{
        maxrects::MaxRects,
        packer::{PackInput, PackOutput, Packer, PlacedSprite},
    },
    imaging::{loader, trim},
    types::{
        atlas::{AtlasFrame, PackedAtlas},
        config::{LayoutConfig, PackMode, SpriteConfig},
        rect::{Rect, Size, SourceRect},
        sprite::Sprite,
    },
};
use fastpack_formats::{
    exporter::{ExportInput, Exporter},
    formats::json_hash::JsonHashExporter,
};
use walkdir::WalkDir;

static IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "bmp", "tga", "webp", "tiff", "tif", "gif",
];

/// Arguments for a single headless pack run.
pub struct PackArgs {
    pub inputs: Vec<PathBuf>,
    pub output_dir: PathBuf,
    pub name: String,
    pub max_width: u32,
    pub max_height: u32,
    pub pack_mode: PackMode,
}

/// Summary produced by a successful pack run.
pub struct PackResult {
    pub sprite_count: usize,
    pub atlas_size: Size,
    pub overflow_count: usize,
    pub texture_bytes: usize,
    pub data_bytes: usize,
    pub texture_path: PathBuf,
    pub data_path: PathBuf,
}

/// Run the full pack pipeline and write output files to disk.
pub fn run_pack(args: PackArgs) -> Result<PackResult> {
    // 1. Collect
    let paths = collect_images(&args.inputs);
    if paths.is_empty() {
        anyhow::bail!("no images found in the specified inputs");
    }

    // 2. Load
    let sprites: Vec<Sprite> = loader::load_many(&paths)
        .into_iter()
        .filter_map(|r| match r {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("warning: {e}");
                None
            }
        })
        .collect();

    if sprites.is_empty() {
        anyhow::bail!("all images failed to load");
    }

    // 3. Trim
    let sprite_cfg = SpriteConfig::default();
    let mut sprites = sprites;
    for s in &mut sprites {
        trim::trim(s, &sprite_cfg);
    }

    // 4. Pack
    let sprite_count = sprites.len();
    let layout = LayoutConfig {
        max_width: args.max_width,
        max_height: args.max_height,
        ..LayoutConfig::default()
    };
    let pack_output: PackOutput = MaxRects::default()
        .pack(PackInput {
            sprites,
            config: layout,
            sprite_config: SpriteConfig::default(),
        })
        .context("packing failed")?;

    let overflow_count = pack_output.overflow.len();

    // 5. Compose
    let atlas_image = compose(&pack_output.placed, &pack_output.atlas_size);

    // 6. Build packed atlas metadata
    let packed = build_packed_atlas(&pack_output, &args.name);

    // 7. Compress
    let compressed = PngCompressor
        .compress(&CompressInput {
            image: &atlas_image,
            pack_mode: args.pack_mode,
        })
        .context("png compression failed")?;

    // 8. Export
    let texture_filename = format!("{}.png", args.name);
    let json_str = JsonHashExporter
        .export(&ExportInput {
            atlas: &packed,
            texture_filename: texture_filename.clone(),
            pixel_format: "RGBA8888".to_string(),
        })
        .context("json export failed")?;

    // 9. Write
    std::fs::create_dir_all(&args.output_dir).context("failed to create output directory")?;
    let texture_path = args.output_dir.join(&texture_filename);
    let data_path = args.output_dir.join(format!("{}.json", args.name));
    std::fs::write(&texture_path, &compressed.data).context("failed to write texture")?;
    std::fs::write(&data_path, json_str.as_bytes()).context("failed to write data file")?;

    Ok(PackResult {
        sprite_count,
        atlas_size: pack_output.atlas_size,
        overflow_count,
        texture_bytes: compressed.data.len(),
        data_bytes: json_str.len(),
        texture_path,
        data_path,
    })
}

fn collect_images(inputs: &[PathBuf]) -> Vec<(PathBuf, String)> {
    let mut paths = Vec::new();
    for input in inputs {
        if input.is_file() {
            if is_image(input) {
                let base = input.parent().unwrap_or(Path::new(""));
                paths.push((input.clone(), file_id(input, base)));
            }
        } else {
            for entry in WalkDir::new(input)
                .sort_by_file_name()
                .into_iter()
                .flatten()
            {
                if entry.file_type().is_file() && is_image(entry.path()) {
                    let id = file_id(entry.path(), input);
                    paths.push((entry.path().to_path_buf(), id));
                }
            }
        }
    }
    paths
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn file_id(path: &Path, base: &Path) -> String {
    let rel = path.strip_prefix(base).unwrap_or(path);
    rel.with_extension("").to_string_lossy().replace('\\', "/")
}

fn compose(placed: &[PlacedSprite], atlas_size: &Size) -> image::DynamicImage {
    let mut canvas = image::DynamicImage::new_rgba8(atlas_size.w, atlas_size.h);
    let canvas_rgba = canvas.as_mut_rgba8().expect("canvas is rgba8");

    for ps in placed {
        let rgba = ps.sprite.image.as_rgba8().expect("sprite is rgba8");
        if ps.placement.rotated {
            let rotated = image::imageops::rotate90(rgba);
            image::imageops::replace(
                canvas_rgba,
                &rotated,
                ps.placement.dest.x as i64,
                ps.placement.dest.y as i64,
            );
        } else {
            image::imageops::replace(
                canvas_rgba,
                rgba,
                ps.placement.dest.x as i64,
                ps.placement.dest.y as i64,
            );
        }
    }

    canvas
}

fn build_packed_atlas(pack_output: &PackOutput, name: &str) -> PackedAtlas {
    let frames: Vec<AtlasFrame> = pack_output
        .placed
        .iter()
        .map(|ps| {
            let dest = &ps.placement.dest;
            let sprite = &ps.sprite;
            let trim_w = sprite.image.width();
            let trim_h = sprite.image.height();

            let sprite_source_size = match &sprite.trim_rect {
                Some(tr) => SourceRect {
                    x: tr.x,
                    y: tr.y,
                    w: trim_w,
                    h: trim_h,
                },
                None => SourceRect {
                    x: 0,
                    y: 0,
                    w: trim_w,
                    h: trim_h,
                },
            };

            AtlasFrame {
                id: ps.placement.sprite_id.clone(),
                frame: Rect {
                    x: dest.x,
                    y: dest.y,
                    w: dest.w,
                    h: dest.h,
                },
                rotated: ps.placement.rotated,
                trimmed: sprite.trim_rect.is_some(),
                sprite_source_size,
                source_size: sprite.original_size,
                polygon: None,
                nine_patch: None,
                pivot: None,
                alias_of: None,
            }
        })
        .collect();

    PackedAtlas {
        frames,
        size: pack_output.atlas_size,
        image: None,
        name: name.to_string(),
        scale: 1.0,
    }
}
