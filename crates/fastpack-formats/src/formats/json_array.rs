use serde::Serialize;

use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter},
};

/// Exports atlas metadata in TexturePacker-compatible JSON Array format.
///
/// Like JSON Hash but `frames` is an array, with each entry carrying a
/// `filename` field instead of using the sprite id as the object key.
pub struct JsonArrayExporter;

impl Exporter for JsonArrayExporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        export_json_array(input)
    }

    fn format_id(&self) -> &'static str {
        "json_array"
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}

fn export_json_array(input: &ExportInput<'_>) -> Result<String, FormatError> {
    let atlas = input.atlas;

    let frames: Vec<JsonFrame> = atlas
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
            alias_of: frame.alias_of.clone(),
        })
        .collect();

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
    frames: Vec<JsonFrame>,
    meta: Meta<'a>,
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
struct Meta<'a> {
    app: &'static str,
    version: &'static str,
    image: &'a str,
    format: &'a str,
    size: WH,
    scale: String,
}
