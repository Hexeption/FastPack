use std::path::{Path, PathBuf};

use anyhow::Result;
use fastpack_core::{
    algorithms::{
        basic::Basic,
        grid::Grid,
        maxrects::MaxRects,
        packer::{PackInput, Packer},
    },
    imaging::{alias::detect_aliases, extrude, loader, trim},
    types::{
        atlas::AtlasFrame,
        config::{AlgorithmConfig, Project},
        rect::{Rect, Size, SourceRect},
        sprite::Sprite,
    },
};
use rayon::prelude::*;
use walkdir::WalkDir;

/// A single packed frame returned to the UI thread.
pub struct FrameInfo {
    pub id: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub alias_of: Option<String>,
}

/// One packed sheet (atlas texture + frame metadata).
pub struct SheetOutput {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<FrameInfo>,
    pub atlas_frames: Vec<AtlasFrame>,
}

/// Data returned to the UI after a successful pack.
pub struct WorkerOutput {
    pub sheets: Vec<SheetOutput>,
    /// Unique sprites packed (excluding aliases).
    pub sprite_count: usize,
    /// Sprites deduplicated as aliases.
    pub alias_count: usize,
    /// Sprites that did not fit (only non-zero when multipack is disabled).
    pub overflow_count: usize,
}

/// Messages sent from the worker thread to the UI thread.
pub enum WorkerMessage {
    Started,
    Progress { done: usize, total: usize },
    Finished(Box<WorkerOutput>),
    Failed(String),
}

static IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "bmp", "tga", "webp", "tiff", "tif", "gif",
];

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

fn collect_images(project: &Project) -> Vec<(PathBuf, String)> {
    let mut paths = Vec::new();
    for source in &project.sources {
        if source.path.is_file() {
            if is_image(&source.path) {
                let base = source.path.parent().unwrap_or(Path::new(""));
                paths.push((source.path.clone(), file_id(&source.path, base)));
            }
        } else {
            for entry in WalkDir::new(&source.path)
                .sort_by_file_name()
                .into_iter()
                .flatten()
            {
                if entry.file_type().is_file() && is_image(entry.path()) {
                    let id = file_id(entry.path(), &source.path);
                    paths.push((entry.path().to_path_buf(), id));
                }
            }
        }
    }
    paths
}

fn build_sheet(
    packer: &dyn Packer,
    sprites: Vec<Sprite>,
    project: &Project,
) -> Result<(SheetOutput, Vec<Sprite>)> {
    let sprite_cfg = &project.config.sprites;
    let pack_output = packer
        .pack(PackInput {
            sprites,
            config: project.config.layout.clone(),
            sprite_config: sprite_cfg.clone(),
        })
        .map_err(|e| anyhow::anyhow!("packing failed: {e}"))?;

    let overflow = pack_output.overflow;

    let mut canvas =
        image::DynamicImage::new_rgba8(pack_output.atlas_size.w, pack_output.atlas_size.h);
    {
        let canvas_rgba = canvas.as_mut_rgba8().expect("canvas is rgba8");
        for ps in &pack_output.placed {
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
    }

    let frames: Vec<FrameInfo> = pack_output
        .placed
        .iter()
        .map(|ps| FrameInfo {
            id: ps.placement.sprite_id.clone(),
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
                id: ps.placement.sprite_id.clone(),
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
                nine_patch: ps.sprite.nine_patch,
                pivot: ps.sprite.pivot,
                alias_of: None,
            }
        })
        .collect();

    let width = pack_output.atlas_size.w;
    let height = pack_output.atlas_size.h;
    let rgba = canvas.into_rgba8().into_raw();

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

/// Run the full pack pipeline for the given project and return raw atlas data.
///
/// Intended to be called from a background thread.
pub fn run_pack(project: &Project) -> Result<WorkerOutput> {
    // 1. Collect
    let paths = collect_images(project);
    if paths.is_empty() {
        anyhow::bail!("no images found in the configured sources");
    }

    // 2. Load (parallel)
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

    // 3. Trim (parallel)
    sprites
        .par_iter_mut()
        .for_each(|s| trim::trim(s, sprite_cfg));

    // 3.5 Extrude (parallel)
    if sprite_cfg.extrude > 0 {
        sprites
            .par_iter_mut()
            .for_each(|s| extrude::extrude(s, sprite_cfg.extrude));
    }

    let sprite_count = sprites.len();

    // 4. Alias detection
    let (base_sprites, base_aliases) = if sprite_cfg.detect_aliases {
        detect_aliases(sprites)
    } else {
        (sprites, Vec::new())
    };
    let alias_count = base_aliases.len();

    // 5. Build packer
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

    // 6. Pack loop (multipack produces multiple sheets)
    let multipack = project.config.output.multipack;
    let mut remaining = base_sprites;
    let mut overflow_count = 0;
    let mut sheets: Vec<SheetOutput> = Vec::new();

    loop {
        let (mut sheet, overflow) = build_sheet(packer.as_ref(), remaining, project)?;
        remaining = overflow;

        // Aliases point into sheet 0; append them there only.
        if sheets.is_empty() {
            let alias_coords: Vec<(u32, u32, u32, u32)> = {
                let frame_id_to_rect: std::collections::HashMap<&str, (u32, u32, u32, u32)> = sheet
                    .frames
                    .iter()
                    .map(|f| (f.id.as_str(), (f.x, f.y, f.w, f.h)))
                    .collect();
                base_aliases
                    .iter()
                    .map(|alias| {
                        let canon = alias.alias_of.as_deref().unwrap_or("");
                        frame_id_to_rect.get(canon).copied().unwrap_or_default()
                    })
                    .collect()
            };

            for (alias, (x, y, w, h)) in base_aliases.iter().zip(alias_coords) {
                sheet.frames.push(FrameInfo {
                    id: alias.id.clone(),
                    x,
                    y,
                    w,
                    h,
                    alias_of: alias.alias_of.clone(),
                });
                sheet.atlas_frames.push(AtlasFrame {
                    id: alias.id.clone(),
                    frame: Rect::new(x, y, w, h),
                    rotated: false,
                    trimmed: false,
                    sprite_source_size: SourceRect {
                        x: 0,
                        y: 0,
                        w: alias.original_size.w,
                        h: alias.original_size.h,
                    },
                    source_size: alias.original_size,
                    polygon: None,
                    nine_patch: alias.nine_patch,
                    pivot: alias.pivot,
                    alias_of: alias.alias_of.clone(),
                });
            }
        }

        sheets.push(sheet);

        if remaining.is_empty() {
            break;
        }
        if !multipack {
            overflow_count = remaining.len();
            break;
        }
    }

    Ok(WorkerOutput {
        sheets,
        sprite_count,
        alias_count,
        overflow_count,
    })
}
