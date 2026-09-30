//! Background pack worker.
//!
//! Runs sprite loading, trimming, packing, and atlas compositing off the main
//! thread. Produces raw RGBA sheet data that the UI converts to base64 PNG for
//! preview, or writes compressed textures + data files to disk on publish.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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
        basic::Basic,
        grid::Grid,
        maxrects::MaxRects,
        packer::{PackInput, Packer},
    },
    imaging::{alias::detect_aliases, bleed, dither, extrude, loader, naming, premultiply, trim},
    types::{
        atlas::{AtlasFrame, PackedAtlas},
        config::{AlgorithmConfig, DataFormat, Project},
        pixel_format::TextureFormat,
        rect::{Rect, Size, SourceRect},
        sprite::Sprite,
    },
};
use fastpack_formats::{
    exporter::{ExportInput, Exporter, SpriteRotation},
    formats::{
        cocos2d::Cocos2dExporter, json_array::JsonArrayExporter, json_hash::JsonHashExporter,
        phaser3::Phaser3Exporter, pixijs::PixiJsExporter,
    },
};
use rayon::prelude::*;
use walkdir::WalkDir;

/// A single packed frame returned to the caller.
pub struct FrameInfo {
    /// Sprite identifier (forward-slash relative path, no extension).
    pub id: String,
    /// Absolute path to the source image.
    pub src_path: String,
    /// X position in the atlas.
    pub x: u32,
    /// Y position in the atlas.
    pub y: u32,
    /// Frame width in pixels.
    pub w: u32,
    /// Frame height in pixels.
    pub h: u32,
    /// Set when this frame is a duplicate of another sprite.
    pub alias_of: Option<String>,
}

/// One packed sheet (atlas texture + frame metadata).
pub struct SheetOutput {
    /// Raw RGBA pixel data for the atlas image.
    pub rgba: Vec<u8>,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Per-frame placement info for the UI overlay.
    pub frames: Vec<FrameInfo>,
    /// Per-frame metadata used by data format exporters.
    pub atlas_frames: Vec<AtlasFrame>,
}

/// Data returned after a successful pack.
pub struct WorkerOutput {
    /// All packed atlas sheets. Multiple sheets when multipack is on.
    pub sheets: Vec<SheetOutput>,
    /// Total number of sprites that were loaded.
    pub sprite_count: usize,
    /// Number of sprites detected as pixel-identical aliases.
    pub alias_count: usize,
    /// Number of sprites that did not fit in any sheet.
    pub overflow_count: usize,
}

/// Supported image file extensions for sprite loading.
static IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "bmp", "tga", "webp", "tiff", "tif", "gif",
];

/// Check whether a file path has a recognised image extension.
fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Walk all project source directories and collect `(path, id)` pairs for images,
/// respecting the exclude list, plus a map from source path to export frame name.
fn collect_images(project: &Project) -> (Vec<(PathBuf, String)>, HashMap<PathBuf, String>) {
    let excludes: std::collections::HashSet<&str> =
        project.config.excludes.iter().map(|s| s.as_str()).collect();
    let sprite_cfg = &project.config.sprites;
    let mut paths = Vec::new();
    let mut names = HashMap::new();
    for source in &project.sources {
        if source.path.is_file() {
            if is_image(&source.path) {
                let base = source.path.parent().unwrap_or(Path::new(""));
                let id = naming::sprite_id(&source.path, base);
                if !excludes.contains(id.as_str()) {
                    names.insert(
                        source.path.clone(),
                        naming::sprite_name(&source.path, base, None, sprite_cfg),
                    );
                    paths.push((source.path.clone(), id));
                }
            }
        } else {
            for entry in WalkDir::new(&source.path)
                .sort_by_file_name()
                .into_iter()
                .flatten()
            {
                if entry.file_type().is_file() && is_image(entry.path()) {
                    let id = naming::sprite_id(entry.path(), &source.path);
                    if !excludes.contains(id.as_str()) {
                        let path = entry.path().to_path_buf();
                        names.insert(
                            path.clone(),
                            naming::sprite_name(
                                &path,
                                &source.path,
                                Some(&source.path),
                                sprite_cfg,
                            ),
                        );
                        paths.push((path, id));
                    }
                }
            }
        }
    }
    (paths, names)
}

/// Return the exporter for the configured data format.
fn select_exporter(data_format: DataFormat) -> Box<dyn Exporter> {
    match data_format {
        DataFormat::JsonArray => Box::new(JsonArrayExporter),
        DataFormat::Phaser3 => Box::new(Phaser3Exporter),
        DataFormat::Pixijs => Box::new(PixiJsExporter),
        DataFormat::JsonHash => Box::new(JsonHashExporter),
        DataFormat::Cocos2d => Box::new(Cocos2dExporter),
    }
}

/// Pack a list of sprites into a single atlas sheet, returning the composited
/// RGBA buffer and any overflow sprites that did not fit.
fn build_sheet(
    packer: &dyn Packer,
    sprites: Vec<Sprite>,
    project: &Project,
    names: &HashMap<PathBuf, String>,
) -> Result<(SheetOutput, Vec<Sprite>)> {
    let sprite_cfg = &project.config.sprites;
    let rotation = select_exporter(project.config.output.data_format).rotation();
    let mut layout = project.config.layout.clone();
    if rotation == SpriteRotation::Unsupported {
        layout.allow_rotation = false;
    }
    let pack_output = packer
        .pack(PackInput {
            sprites,
            config: layout,
            sprite_config: sprite_cfg.clone(),
        })
        .map_err(|e| anyhow::anyhow!("packing failed: {e}"))?;

    let overflow = pack_output.overflow;

    // Packer guarantees non-overlapping placements, so parallel writes are sound.
    let aw = pack_output.atlas_size.w as usize;
    let ah = pack_output.atlas_size.h as usize;
    let mut canvas_raw = vec![0u8; aw * ah * 4];
    let buf_ptr = canvas_raw.as_mut_ptr() as usize;
    let buf_stride = aw;

    pack_output.placed.par_iter().for_each(move |ps| {
        let dx = ps.placement.dest.x as usize;
        let dy = ps.placement.dest.y as usize;
        let dw = ps.placement.dest.w as usize;
        let dh = ps.placement.dest.h as usize;
        let rgba = ps.sprite.image.as_rgba8().expect("sprite is rgba8");
        let dst = buf_ptr as *mut u8;

        if ps.placement.rotated {
            let rotated = match rotation {
                SpriteRotation::CounterClockwise => image::imageops::rotate270(rgba),
                _ => image::imageops::rotate90(rgba),
            };
            let src_raw = rotated.as_raw();
            for row in 0..dh {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        src_raw.as_ptr().add(row * dw * 4),
                        dst.add(((dy + row) * buf_stride + dx) * 4),
                        dw * 4,
                    );
                }
            }
        } else {
            let src_raw = rgba.as_raw();
            let src_stride = rgba.width() as usize * 4;
            for row in 0..dh {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        src_raw.as_ptr().add(row * src_stride),
                        dst.add(((dy + row) * buf_stride + dx) * 4),
                        dw * 4,
                    );
                }
            }
        }
    });

    let frames: Vec<FrameInfo> = pack_output
        .placed
        .iter()
        .map(|ps| FrameInfo {
            id: ps.placement.sprite_id.clone(),
            src_path: ps.sprite.source_path.to_string_lossy().into_owned(),
            x: ps.placement.dest.x,
            y: ps.placement.dest.y,
            w: ps.placement.dest.w,
            h: ps.placement.dest.h,
            alias_of: None,
        })
        .collect();

    let atlas_frames: Vec<AtlasFrame> = pack_output
        .placed
        .iter()
        .map(|ps| {
            let trimmed = ps.sprite.trim_rect.is_some();
            let sss = ps.sprite.trim_rect.unwrap_or(SourceRect {
                x: 0,
                y: 0,
                w: ps.sprite.original_size.w,
                h: ps.sprite.original_size.h,
            });
            AtlasFrame {
                id: names
                    .get(&ps.sprite.source_path)
                    .cloned()
                    .unwrap_or_else(|| ps.placement.sprite_id.clone()),
                frame: Rect::new(
                    ps.placement.dest.x,
                    ps.placement.dest.y,
                    ps.placement.dest.w,
                    ps.placement.dest.h,
                ),
                rotated: ps.placement.rotated,
                trimmed,
                sprite_source_size: sss,
                source_size: ps.sprite.original_size,
                polygon: ps.sprite.polygon.clone(),
                extrude: ps.sprite.extrude,
                nine_patch: ps.sprite.nine_patch,
                pivot: ps.sprite.pivot,
                alias_of: None,
            }
        })
        .collect();

    let width = pack_output.atlas_size.w;
    let height = pack_output.atlas_size.h;
    let rgba = canvas_raw;

    Ok((
        SheetOutput {
            rgba,
            width,
            height,
            frames,
            atlas_frames,
        },
        overflow,
    ))
}

/// Run the full pack pipeline and return raw atlas data.
///
/// Intended to be called from a background thread.
pub fn run_pack(project: &Project) -> Result<WorkerOutput> {
    let n = std::thread::available_parallelism()
        .map(|p| p.get().saturating_sub(2).max(1))
        .unwrap_or(1);
    rayon::ThreadPoolBuilder::new()
        .num_threads(n)
        .build()
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .install(|| run_pack_impl(project))
}

/// Internal pack implementation run inside a dedicated rayon thread pool.
fn run_pack_impl(project: &Project) -> Result<WorkerOutput> {
    let (paths, names) = collect_images(project);
    if paths.is_empty() {
        anyhow::bail!("no images found in the configured sources");
    }

    let mut sprites: Vec<Sprite> = paths
        .par_iter()
        .filter_map(|(path, id)| match loader::load(path, id.clone()) {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::warn!("failed to load {}: {e}", path.display());
                None
            }
        })
        .collect();
    if sprites.is_empty() {
        anyhow::bail!("all images failed to load");
    }

    let sprite_cfg = &project.config.sprites;

    sprites
        .par_iter_mut()
        .for_each(|s| trim::trim(s, sprite_cfg));

    if sprite_cfg.extrude > 0 {
        sprites
            .par_iter_mut()
            .for_each(|s| extrude::extrude(s, sprite_cfg.extrude));
    }

    if sprite_cfg.alpha_bleed {
        sprites.par_iter_mut().for_each(bleed::alpha_bleed);
    }

    let sprite_count = sprites.len();

    let (base_sprites, base_aliases) = if sprite_cfg.detect_aliases {
        detect_aliases(sprites)
    } else {
        (sprites, Vec::new())
    };
    let alias_count = base_aliases.len();

    let packer: Box<dyn Packer> = match &project.config.algorithm {
        AlgorithmConfig::Grid {
            cell_width,
            cell_height,
        } => Box::new(Grid {
            cell_width: if *cell_width == 0 {
                None
            } else {
                Some(*cell_width)
            },
            cell_height: if *cell_height == 0 {
                None
            } else {
                Some(*cell_height)
            },
        }),
        AlgorithmConfig::Basic => Box::new(Basic),
        AlgorithmConfig::MaxRects { heuristic } => Box::new(MaxRects {
            heuristic: *heuristic,
        }),
        AlgorithmConfig::Polygon => Box::new(MaxRects::default()),
    };

    let multipack = project.config.output.multipack;
    let mut remaining = base_sprites;
    let mut overflow_count = 0;
    let mut sheets: Vec<SheetOutput> = Vec::new();

    loop {
        let (sheet, overflow) = build_sheet(packer.as_ref(), remaining, project, &names)?;
        remaining = overflow;
        sheets.push(sheet);

        if remaining.is_empty() {
            break;
        }
        if !multipack {
            overflow_count = remaining.len();
            break;
        }
    }

    // Build a map from canonical sprite ID to its sheet index, atlas placement,
    // and export name. Done after all sheets are packed so multipack aliases find
    // their canonical regardless of which sheet it landed on. Keyed by internal id
    // (`FrameInfo::id`) because `alias_of` holds ids, not export names.
    type CanonEntry = (usize, u32, u32, u32, u32, bool, String);
    let canon_map: HashMap<String, CanonEntry> = {
        let mut m = HashMap::new();
        for (sheet_idx, sheet) in sheets.iter().enumerate() {
            for (fi, af) in sheet.frames.iter().zip(&sheet.atlas_frames) {
                m.insert(
                    fi.id.clone(),
                    (
                        sheet_idx,
                        af.frame.x,
                        af.frame.y,
                        af.frame.w,
                        af.frame.h,
                        af.rotated,
                        af.id.clone(),
                    ),
                );
            }
        }
        m
    };

    // Collect alias frames grouped by their target sheet index.
    let mut pending: Vec<(usize, FrameInfo, AtlasFrame)> = Vec::new();
    for alias in &base_aliases {
        let canon_id = alias.alias_of.as_deref().unwrap_or("");
        let Some((sheet_idx, x, y, w, h, rotated, canon_name)) = canon_map.get(canon_id) else {
            continue;
        };
        let (sheet_idx, x, y, w, h, rotated) = (*sheet_idx, *x, *y, *w, *h, *rotated);
        let trimmed = alias.trim_rect.is_some();
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
        pending.push((
            sheet_idx,
            FrameInfo {
                id: alias.id.clone(),
                src_path: alias.source_path.to_string_lossy().into_owned(),
                x,
                y,
                w,
                h,
                alias_of: alias.alias_of.clone(),
            },
            AtlasFrame {
                id: names
                    .get(&alias.source_path)
                    .cloned()
                    .unwrap_or_else(|| alias.id.clone()),
                frame: Rect::new(x, y, w, h),
                rotated,
                trimmed,
                sprite_source_size,
                source_size: alias.original_size,
                polygon: alias.polygon.clone(),
                extrude: alias.extrude,
                nine_patch: alias.nine_patch,
                pivot: alias.pivot,
                alias_of: Some(canon_name.clone()),
            },
        ));
    }
    for (sheet_idx, frame_info, atlas_frame) in pending {
        sheets[sheet_idx].frames.push(frame_info);
        sheets[sheet_idx].atlas_frames.push(atlas_frame);
    }

    Ok(WorkerOutput {
        sheets,
        sprite_count,
        alias_count,
        overflow_count,
    })
}

/// Write packed sheets to disk using the project's output configuration.
///
/// Returns `(file_count, resolved_output_dir)` on success.
/// `project_path` is the path of the saved `.fpsheet` file; relative output
/// directories are resolved against its parent.
pub fn write_output(
    output: &WorkerOutput,
    project: &Project,
    project_path: Option<&Path>,
) -> Result<(usize, PathBuf)> {
    let out_cfg = &project.config.output;

    let out_dir = if out_cfg.directory.as_os_str().is_empty() {
        anyhow::bail!("output directory is not configured");
    } else if out_cfg.directory.is_absolute() {
        out_cfg.directory.clone()
    } else if let Some(pp) = project_path {
        pp.parent()
            .unwrap_or(Path::new("."))
            .join(&out_cfg.directory)
    } else {
        out_cfg.directory.clone()
    };

    std::fs::create_dir_all(&out_dir).context("failed to create output directory")?;

    let compressor: Box<dyn Compressor> = match out_cfg.texture_format {
        TextureFormat::Png => Box::new(PngCompressor),
        TextureFormat::Jpeg => Box::new(JpegCompressor),
        TextureFormat::WebP => Box::new(WebpCompressor),
        TextureFormat::Dxt1 => Box::new(Dxt1Compressor),
        TextureFormat::Dxt5 => Box::new(Dxt5Compressor),
        _ => Box::new(PngCompressor),
    };

    let exporter = select_exporter(out_cfg.data_format);
    let data_ext = exporter.file_extension();

    let tex_ext = out_cfg.texture_format.extension();
    let base_name = &out_cfg.name;
    let pack_mode = project.config.layout.pack_mode;

    let mut atlases: Vec<PackedAtlas> = Vec::new();
    let mut tex_filenames: Vec<String> = Vec::new();
    let mut file_count = 0usize;

    // Remove stale output files from previous runs.
    if let Ok(entries) = std::fs::read_dir(&out_dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let fname = fname.to_string_lossy();
            let is_stale_tex = fname.starts_with(&format!("{base_name}-"))
                && fname.ends_with(&format!(".{tex_ext}"));
            let is_stale_data = (fname.starts_with(&format!("{base_name}-"))
                && fname.ends_with(&format!(".{data_ext}")))
                || fname == format!("{base_name}.{data_ext}");
            if is_stale_tex || is_stale_data {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    for (i, sheet) in output.sheets.iter().enumerate() {
        use image::{DynamicImage, ImageBuffer, Rgba};

        let img: ImageBuffer<Rgba<u8>, _> =
            ImageBuffer::from_raw(sheet.width, sheet.height, sheet.rgba.clone())
                .context("invalid rgba buffer")?;
        let img = DynamicImage::ImageRgba8(img);

        let img = if out_cfg.premultiply_alpha {
            premultiply::premultiply(&img)
        } else {
            img
        };

        let img = dither::dither(&img, out_cfg.pixel_format);

        let compressed = compressor
            .compress(&CompressInput {
                image: &img,
                pack_mode,
                quality: out_cfg.quality,
            })
            .context("compression failed")?;

        let tex_filename = format!("{base_name}-{i}.{tex_ext}");
        let tex_path = out_dir.join(&tex_filename);
        std::fs::write(&tex_path, &compressed.data).context("failed to write texture")?;
        file_count += 1;

        atlases.push(PackedAtlas {
            frames: sheet.atlas_frames.clone(),
            size: Size {
                w: sheet.width,
                h: sheet.height,
            },
            image: None,
            name: base_name.clone(),
            scale: 1.0,
        });
        tex_filenames.push(tex_filename);
    }

    let export_inputs: Vec<ExportInput<'_>> = atlases
        .iter()
        .zip(&tex_filenames)
        .map(|(atlas, fname)| ExportInput {
            atlas,
            texture_filename: fname.clone(),
            pixel_format: out_cfg.pixel_format.to_string(),
            hide_name: out_cfg.hide_name,
        })
        .collect();

    match exporter.combine(&export_inputs) {
        Some(result) => {
            let content = result.context("combined export failed")?;
            let data_path = out_dir.join(format!("{base_name}.{data_ext}"));
            std::fs::write(&data_path, content.as_bytes()).context("failed to write data file")?;
            file_count += 1;
        }
        None => {
            for (i, input) in export_inputs.iter().enumerate() {
                let content = exporter.export(input).context("export failed")?;
                let stem = if output.sheets.len() == 1 {
                    base_name.clone()
                } else {
                    format!("{base_name}-{i}")
                };
                let data_path = out_dir.join(format!("{stem}.{data_ext}"));
                std::fs::write(&data_path, content.as_bytes())
                    .context("failed to write data file")?;
                file_count += 1;
            }
        }
    }

    Ok((file_count, out_dir))
}
