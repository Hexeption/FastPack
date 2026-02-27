use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use fastpack_compress::{
    backends::{
        dxt::{Dxt1Compressor, Dxt5Compressor},
        jpeg::JpegCompressor,
        png::PngCompressor,
        webp::WebpCompressor,
    },
    compressor::{CompressInput, Compressor},
};
use fastpack_core::{
    algorithms::{
        maxrects::MaxRects,
        packer::{PackInput, PackOutput, Packer, PlacedSprite},
    },
    imaging::{alias::detect_aliases, dither, extrude, loader, premultiply, scale, trim},
    types::{
        atlas::{AtlasFrame, PackedAtlas},
        config::{DataFormat, LayoutConfig, ScaleVariant, SpriteConfig, SpriteOverride},
        pixel_format::{PixelFormat, TextureFormat},
        rect::{Point, Rect, Size, SourceRect},
        sprite::Sprite,
    },
};
use fastpack_formats::{
    exporter::{ExportInput, Exporter},
    formats::{
        json_array::JsonArrayExporter, json_hash::JsonHashExporter, phaser3::Phaser3Exporter,
        pixijs::PixiJsExporter,
    },
};
use indicatif::{MultiProgress, ParallelProgressIterator};
use rayon::prelude::*;
use std::collections::HashMap;
use walkdir::WalkDir;

use crate::progress;

static IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "bmp", "tga", "webp", "tiff", "tif", "gif",
];

/// Arguments for a single headless pack run.
pub struct PackArgs {
    /// Input directories or individual image files to pack.
    pub inputs: Vec<PathBuf>,
    /// Directory where atlas texture and data files are written.
    pub output_dir: PathBuf,
    /// Base name for output files (no extension).
    pub name: String,
    /// Full layout configuration (dimensions, padding, constraints, rotation, etc.).
    pub layout: LayoutConfig,
    /// Sprite pre-processing options (trim, extrude, alias detection, etc.).
    pub sprite_config: SpriteConfig,
    /// Emit additional sheets when sprites overflow the first atlas.
    pub multipack: bool,
    /// Default pivot written to data files. `None` omits the pivot field entirely.
    pub default_pivot: Option<Point>,
    /// Per-sprite metadata (pivot, nine-patch) read from the project file.
    pub sprite_overrides: Vec<SpriteOverride>,
    /// Scale variants to produce. An empty list is treated as a single @1x variant.
    pub variants: Vec<ScaleVariant>,
    /// Output data serialization format.
    pub data_format: DataFormat,
    /// Output texture container / hardware compression format.
    pub texture_format: TextureFormat,
    /// Pixel-level bit depth. Dithering is applied when this is not Rgba8888.
    pub pixel_format: PixelFormat,
    /// Premultiply RGB channels by alpha before compression.
    pub premultiply_alpha: bool,
    /// Sprite IDs excluded from packing.
    pub excludes: Vec<String>,
}

/// Per-sheet output produced by a pack run.
pub struct SheetResult {
    /// Pixel dimensions of the packed atlas texture.
    pub atlas_size: Size,
    /// Compressed texture file size in bytes.
    pub texture_bytes: usize,
    /// Data file size in bytes.
    pub data_bytes: usize,
    /// Path to the written texture file.
    pub texture_path: PathBuf,
    /// Path to the written data file.
    pub data_path: PathBuf,
}

/// Summary produced by a successful pack run.
pub struct PackResult {
    /// Total number of unique sprites packed (excluding aliases).
    pub sprite_count: usize,
    /// Number of sprites deduplicated as aliases of another sprite.
    pub alias_count: usize,
    /// Sprites that did not fit on any sheet (only non-zero when multipack is disabled).
    pub overflow_count: usize,
    /// One entry per output sheet across all scale variants.
    pub sheets: Vec<SheetResult>,
}

/// Run the full pack pipeline and write output files to disk.
pub fn run_pack(args: PackArgs) -> Result<PackResult> {
    let mp = MultiProgress::new();

    // 1. Collect
    let mut paths = collect_images(&args.inputs);
    if !args.excludes.is_empty() {
        let excludes: std::collections::HashSet<&str> =
            args.excludes.iter().map(|s| s.as_str()).collect();
        paths.retain(|(_, id)| !excludes.contains(id.as_str()));
    }
    if paths.is_empty() {
        anyhow::bail!("no images found in the specified inputs");
    }

    // 2. Load
    let load_bar = progress::count_bar(&mp, paths.len() as u64, "Loading ");
    let load_results: Vec<_> = paths
        .par_iter()
        .map(|(path, id)| loader::load(path, id.clone()))
        .progress_with(load_bar.clone())
        .collect();
    load_bar.finish_and_clear();

    let sprites: Vec<Sprite> = load_results
        .into_iter()
        .filter_map(|r| match r {
            Ok(s) => Some(s),
            Err(e) => {
                let _ = mp.println(format!("warning: {e}"));
                None
            }
        })
        .collect();

    if sprites.is_empty() {
        anyhow::bail!("all images failed to load");
    }

    // 3. Trim
    let sprite_cfg = &args.sprite_config;
    let mut sprites = sprites;
    for s in &mut sprites {
        trim::trim(s, sprite_cfg);
    }

    // 3.5. Extrude
    if sprite_cfg.extrude > 0 {
        for s in &mut sprites {
            extrude::extrude(s, sprite_cfg.extrude);
        }
    }

    let sprite_count = sprites.len();

    // 3.75. Apply per-sprite overrides from the project file.
    if !args.sprite_overrides.is_empty() {
        let overrides: HashMap<&str, &SpriteOverride> = args
            .sprite_overrides
            .iter()
            .map(|ov| (ov.id.as_str(), ov))
            .collect();
        for sprite in &mut sprites {
            if let Some(ov) = overrides.get(sprite.id.as_str()) {
                if ov.pivot.is_some() {
                    sprite.pivot = ov.pivot;
                }
                if ov.nine_patch.is_some() {
                    sprite.nine_patch = ov.nine_patch;
                }
            }
        }
    }

    // 4. Alias detection
    let (base_sprites, base_aliases) = if sprite_cfg.detect_aliases {
        detect_aliases(sprites)
    } else {
        (sprites, Vec::new())
    };
    let alias_count = base_aliases.len();

    let layout = args.layout;

    std::fs::create_dir_all(&args.output_dir).context("failed to create output directory")?;

    let effective_variants: Vec<ScaleVariant> = if args.variants.is_empty() {
        vec![ScaleVariant::default()]
    } else {
        args.variants.clone()
    };

    // 5–10. Variant loop: for each scale variant, run the full pack+compress+export pipeline.
    let mut all_sheets: Vec<SheetResult> = Vec::new();
    let mut overflow_count = 0;

    for variant in &effective_variants {
        // Scale sprites and aliases for this variant.
        let variant_sprites: Vec<Sprite> = if (variant.scale - 1.0).abs() < f32::EPSILON {
            base_sprites.clone()
        } else {
            base_sprites
                .iter()
                .map(|s| scale::scale_sprite(s, variant.scale, variant.scale_mode))
                .collect::<anyhow::Result<Vec<_>>>()
                .context("sprite scaling failed")?
        };
        let variant_aliases: Vec<Sprite> = if (variant.scale - 1.0).abs() < f32::EPSILON {
            base_aliases.clone()
        } else {
            base_aliases
                .iter()
                .map(|s| scale::scale_sprite(s, variant.scale, variant.scale_mode))
                .collect::<anyhow::Result<Vec<_>>>()
                .context("alias scaling failed")?
        };

        // Vectors accumulate per-sheet data across the inner multipack loop so
        // data files can be written after all sheets are known (required by formats
        // like Phaser 3 that combine all sheets into one JSON file).
        let mut variant_atlases: Vec<PackedAtlas> = Vec::new();
        let mut variant_tex_filenames: Vec<String> = Vec::new();
        let variant_sheet_start = all_sheets.len();

        // Inner multipack loop for this variant.
        let mut remaining = variant_sprites;
        let mut variant_sheet_index = 0usize;

        loop {
            // 5. Pack
            let pack_pb = progress::spinner(&mp, "Packing...");
            let pack_output: PackOutput = MaxRects::default()
                .pack(PackInput {
                    sprites: remaining,
                    config: layout.clone(),
                    sprite_config: sprite_cfg.clone(),
                })
                .context("packing failed")?;
            pack_pb.finish_and_clear();

            // 6. Compose
            let atlas_image = compose(&pack_output.placed, &pack_output.atlas_size);

            // 6.25. Premultiply alpha if requested.
            let atlas_image = if args.premultiply_alpha {
                premultiply::premultiply(&atlas_image)
            } else {
                atlas_image
            };

            // 6.5. Dither to target pixel format (no-op for Rgba8888).
            let atlas_image = dither::dither(&atlas_image, args.pixel_format);

            // 7. Build packed atlas metadata (variant aliases only on the first sheet).
            let sheet_aliases = if variant_sheet_index == 0 {
                &variant_aliases[..]
            } else {
                &[]
            };
            let packed =
                build_packed_atlas(&pack_output, sheet_aliases, &args.name, args.default_pivot);

            // 8. Compress
            let compress_pb = progress::spinner(&mp, "Compressing...");
            let compressor = select_compressor(args.texture_format);
            let compressed = compressor
                .compress(&CompressInput {
                    image: &atlas_image,
                    pack_mode: layout.pack_mode,
                    quality: 95,
                })
                .context("compression failed")?;
            compress_pb.finish_and_clear();

            // 9. Write texture; accumulate atlas for deferred data export.
            let tex_ext = args.texture_format.extension();
            let (texture_filename, _) =
                sheet_filename(&args.name, &variant.suffix, variant_sheet_index, tex_ext);
            let texture_path = args.output_dir.join(&texture_filename);
            std::fs::write(&texture_path, &compressed.data).context("failed to write texture")?;

            let atlas_size = pack_output.atlas_size;
            all_sheets.push(SheetResult {
                atlas_size,
                texture_bytes: compressed.data.len(),
                data_bytes: 0,
                texture_path,
                data_path: PathBuf::new(),
            });
            variant_atlases.push(packed);
            variant_tex_filenames.push(texture_filename);

            // Extract overflow last so all pack_output borrows above are satisfied.
            remaining = pack_output.overflow;
            variant_sheet_index += 1;

            if remaining.is_empty() {
                break;
            }
            if !args.multipack {
                overflow_count = remaining.len();
                break;
            }
        }

        // 10. Export data files for this variant.
        let exporter = select_exporter(args.data_format);
        let export_inputs: Vec<ExportInput<'_>> = variant_atlases
            .iter()
            .zip(&variant_tex_filenames)
            .map(|(atlas, fname)| ExportInput {
                atlas,
                texture_filename: fname.clone(),
                pixel_format: args.pixel_format.to_string(),
            })
            .collect();

        let n = variant_atlases.len();
        match exporter.combine(&export_inputs) {
            Some(result) => {
                let content = result.context("combined data export failed")?;
                let base = format!("{}{}", args.name, variant.suffix);
                let data_path = args.output_dir.join(format!("{base}.json"));
                std::fs::write(&data_path, content.as_bytes())
                    .context("failed to write data file")?;
                for i in 0..n {
                    all_sheets[variant_sheet_start + i].data_path = data_path.clone();
                }
                all_sheets[variant_sheet_start].data_bytes = content.len();
            }
            None => {
                for (i, input) in export_inputs.iter().enumerate() {
                    let content = exporter.export(input).context("data export failed")?;
                    let (_, base_name) = sheet_filename(
                        &args.name,
                        &variant.suffix,
                        i,
                        args.texture_format.extension(),
                    );
                    let data_path = args.output_dir.join(format!("{base_name}.json"));
                    std::fs::write(&data_path, content.as_bytes())
                        .context("failed to write data file")?;
                    all_sheets[variant_sheet_start + i].data_path = data_path;
                    all_sheets[variant_sheet_start + i].data_bytes = content.len();
                }
            }
        }
    }

    Ok(PackResult {
        sprite_count,
        alias_count,
        overflow_count,
        sheets: all_sheets,
    })
}

/// Returns `(texture_filename, base_name_without_ext)` for a sheet index.
/// Sheet 0 uses `<name><suffix>.<ext>`; subsequent sheets append the index.
fn sheet_filename(name: &str, suffix: &str, index: usize, ext: &str) -> (String, String) {
    if index == 0 {
        (format!("{name}{suffix}.{ext}"), format!("{name}{suffix}"))
    } else {
        (
            format!("{name}{suffix}{index}.{ext}"),
            format!("{name}{suffix}{index}"),
        )
    }
}

fn select_exporter(data_format: DataFormat) -> Box<dyn Exporter> {
    match data_format {
        DataFormat::JsonArray => Box::new(JsonArrayExporter),
        DataFormat::Phaser3 => Box::new(Phaser3Exporter),
        DataFormat::Pixijs => Box::new(PixiJsExporter),
        DataFormat::JsonHash => Box::new(JsonHashExporter),
    }
}

fn select_compressor(texture_format: TextureFormat) -> Box<dyn Compressor> {
    match texture_format {
        TextureFormat::Png => Box::new(PngCompressor),
        TextureFormat::Jpeg => Box::new(JpegCompressor),
        TextureFormat::WebP => Box::new(WebpCompressor),
        TextureFormat::Dxt1 => Box::new(Dxt1Compressor),
        TextureFormat::Dxt5 => Box::new(Dxt5Compressor),
        // Formats requiring external toolchains fall back to PNG.
        _ => Box::new(PngCompressor),
    }
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

fn build_packed_atlas(
    pack_output: &PackOutput,
    aliases: &[Sprite],
    name: &str,
    default_pivot: Option<Point>,
) -> PackedAtlas {
    let mut frames: Vec<AtlasFrame> = pack_output
        .placed
        .iter()
        .map(|ps| {
            let dest = &ps.placement.dest;
            let sprite = &ps.sprite;

            let sprite_source_size = match &sprite.trim_rect {
                Some(tr) => SourceRect {
                    x: tr.x,
                    y: tr.y,
                    w: tr.w,
                    h: tr.h,
                },
                None => SourceRect {
                    x: 0,
                    y: 0,
                    w: sprite.original_size.w,
                    h: sprite.original_size.h,
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
                polygon: sprite.polygon.clone(),
                nine_patch: sprite.nine_patch,
                pivot: sprite.pivot.or(default_pivot),
                alias_of: None,
            }
        })
        .collect();

    // Build id → index map so alias frames can reference the canonical atlas rect.
    let frame_by_id: std::collections::HashMap<String, usize> = frames
        .iter()
        .enumerate()
        .map(|(i, f)| (f.id.clone(), i))
        .collect();

    for alias in aliases {
        let canon_id = alias.alias_of.as_deref().unwrap_or("");
        if let Some(&ci) = frame_by_id.get(canon_id) {
            let (canon_frame, canon_rotated) = (frames[ci].frame, frames[ci].rotated);
            let sprite_source_size = match &alias.trim_rect {
                Some(tr) => SourceRect {
                    x: tr.x,
                    y: tr.y,
                    w: tr.w,
                    h: tr.h,
                },
                None => SourceRect {
                    x: 0,
                    y: 0,
                    w: alias.original_size.w,
                    h: alias.original_size.h,
                },
            };
            frames.push(AtlasFrame {
                id: alias.id.clone(),
                frame: canon_frame,
                rotated: canon_rotated,
                trimmed: alias.trim_rect.is_some(),
                sprite_source_size,
                source_size: alias.original_size,
                polygon: alias.polygon.clone(),
                nine_patch: alias.nine_patch,
                pivot: alias.pivot.or(default_pivot),
                alias_of: alias.alias_of.clone(),
            });
        }
    }

    PackedAtlas {
        frames,
        size: pack_output.atlas_size,
        image: None,
        name: name.to_string(),
        scale: 1.0,
    }
}
