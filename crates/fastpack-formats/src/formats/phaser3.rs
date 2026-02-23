use serde::Serialize;

use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter},
};

/// Exports atlas metadata in Phaser 3 multi-atlas format.
///
/// The output is a single JSON file with a `textures` array. Each entry in the
/// array represents one atlas sheet and carries its own `image`, `size`, and
/// `frames` array. Single-sheet packs produce a `textures` array with one entry.
///
/// This format is understood by Phaser 3's `scene.load.multiatlas()` loader.
pub struct Phaser3Exporter;

impl Exporter for Phaser3Exporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        build_output(std::slice::from_ref(input))
    }

    fn combine(&self, inputs: &[ExportInput<'_>]) -> Option<Result<String, FormatError>> {
        Some(build_output(inputs))
    }

    fn format_id(&self) -> &'static str {
        "phaser3"
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}

fn build_output(inputs: &[ExportInput<'_>]) -> Result<String, FormatError> {
    let textures: Vec<TextureEntry> = inputs
        .iter()
        .map(|inp| TextureEntry {
            image: inp.texture_filename.clone(),
            format: inp.pixel_format.clone(),
            size: WH {
                w: inp.atlas.size.w,
                h: inp.atlas.size.h,
            },
            scale: inp.atlas.scale,
            frames: build_frames(inp),
        })
        .collect();

    let output = Output {
        textures,
        meta: Meta {
            app: "FastPack",
            version: "3.0",
        },
    };

    serde_json::to_string_pretty(&output).map_err(FormatError::Json)
}

fn build_frames(input: &ExportInput<'_>) -> Vec<JsonFrame> {
    input
        .atlas
        .frames
        .iter()
        .map(|frame| JsonFrame {
            filename: frame.id.clone(),
            frame: URect {
                x: frame.frame.x,
                y: frame.frame.y,
                w: frame.frame.w,
                h: frame.frame.h,
            },
            rotated: frame.rotated,
            trimmed: frame.trimmed,
            sprite_source_size: IRect {
                x: frame.sprite_source_size.x,
                y: frame.sprite_source_size.y,
                w: frame.sprite_source_size.w,
                h: frame.sprite_source_size.h,
            },
            source_size: WH {
                w: frame.source_size.w,
                h: frame.source_size.h,
            },
        })
        .collect()
}

#[derive(Serialize)]
struct Output {
    textures: Vec<TextureEntry>,
    meta: Meta,
}

#[derive(Serialize)]
struct TextureEntry {
    image: String,
    format: String,
    size: WH,
    scale: f32,
    frames: Vec<JsonFrame>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonFrame {
    filename: String,
    frame: URect,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: IRect,
    source_size: WH,
}

#[derive(Serialize)]
struct URect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct IRect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct WH {
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct Meta {
    app: &'static str,
    version: &'static str,
}
