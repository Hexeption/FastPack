use serde::Serialize;

use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter},
};

/// Exports atlas metadata in TexturePacker-compatible JSON Hash format.
///
/// Each frame is a key in the top-level `"frames"` object. The frame key is
/// the sprite ID (relative path without extension).
pub struct JsonHashExporter;

impl Exporter for JsonHashExporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        export_json_hash(input)
    }

    fn format_id(&self) -> &'static str {
        "json_hash"
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}

fn export_json_hash(input: &ExportInput<'_>) -> Result<String, FormatError> {
    let atlas = input.atlas;

    let mut frames = serde_json::Map::new();
    for frame in &atlas.frames {
        let f = JsonFrame {
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
            pivot: frame.pivot.map(|p| XY { x: p.x, y: p.y }),
            alias_of: frame.alias_of.clone(),
        };
        frames.insert(frame.id.clone(), serde_json::to_value(f)?);
    }

    let output = Output {
        frames,
        meta: Meta {
            app: "FastPack",
            version: "1.0",
            image: &input.texture_filename,
            format: &input.pixel_format,
            size: WH {
                w: atlas.size.w,
                h: atlas.size.h,
            },
            scale: atlas.scale.to_string(),
        },
    };

    serde_json::to_string_pretty(&output).map_err(FormatError::Json)
}

#[derive(Serialize)]
struct Output<'a> {
    frames: serde_json::Map<String, serde_json::Value>,
    meta: Meta<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonFrame {
    frame: URect,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: IRect,
    source_size: WH,
    #[serde(skip_serializing_if = "Option::is_none")]
    pivot: Option<XY>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias_of: Option<String>,
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
struct XY {
    x: f32,
    y: f32,
}

#[derive(Serialize)]
struct Meta<'a> {
    app: &'static str,
    version: &'static str,
    image: &'a str,
    format: &'a str,
    size: WH,
    scale: String,
}
