use std::path::{Path, PathBuf};

use anyhow::Result;
use fastpack_core::{
    algorithms::{
        maxrects::MaxRects,
        packer::{PackInput, Packer},
    },
    imaging::{alias::detect_aliases, extrude, loader, trim},
    types::{config::Project, sprite::Sprite},
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

/// Data returned to the UI after a successful pack.
pub struct WorkerOutput {
    /// Raw RGBA8888 pixel bytes.
    pub rgba: Vec<u8>,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Unique sprites packed (excluding aliases).
    pub sprite_count: usize,
    /// Sprites deduplicated as aliases.
    pub alias_count: usize,
    /// Sprites that did not fit.
    pub overflow_count: usize,
    /// Frame list for the sprite list and atlas preview.
    pub frames: Vec<FrameInfo>,
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

    // 5. Pack
    let pack_output = MaxRects::default()
        .pack(PackInput {
            sprites: base_sprites,
            config: project.config.layout.clone(),
            sprite_config: sprite_cfg.clone(),
        })
        .map_err(|e| anyhow::anyhow!("packing failed: {e}"))?;

    let overflow_count = pack_output.overflow.len();

    // 6. Compose atlas image
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

    // 7. Build frame list
    let frame_id_to_rect: std::collections::HashMap<&str, (u32, u32, u32, u32)> = pack_output
        .placed
        .iter()
        .map(|ps| {
            let d = &ps.placement.dest;
            (ps.placement.sprite_id.as_str(), (d.x, d.y, d.w, d.h))
        })
        .collect();

    let mut frames: Vec<FrameInfo> = pack_output
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

    for alias in &base_aliases {
        let canon = alias.alias_of.as_deref().unwrap_or("");
        let (x, y, w, h) = frame_id_to_rect.get(canon).copied().unwrap_or_default();
        frames.push(FrameInfo {
            id: alias.id.clone(),
            x,
            y,
            w,
            h,
            alias_of: alias.alias_of.clone(),
        });
    }

    let width = pack_output.atlas_size.w;
    let height = pack_output.atlas_size.h;
    let rgba = canvas.into_rgba8().into_raw();

    Ok(WorkerOutput {
        rgba,
        width,
        height,
        sprite_count,
        alias_count,
        overflow_count,
        frames,
    })
}
