use serde::Serialize;

use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter, SpriteRotation},
    smartupdate,
    text::{image_name, upright_size},
};

/// Exports atlas metadata as a Godot 4 `.tpsheet`, the JSON format read by
/// CodeAndWeb's TexturePacker importer plugin for Godot.
///
/// Each sheet becomes an entry in `textures`, so multipack output is combined
/// into one file. Sprites become `AtlasTexture` resources: `region` is the rect
/// in the texture and `margin` restores trimmed transparent borders. Godot's
/// `AtlasTexture` cannot represent rotated regions, so rotation is disabled for
/// this format. Filenames carry a `.png` suffix like TexturePacker's output.
pub struct GodotExporter;

impl Exporter for GodotExporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        build_output(std::slice::from_ref(input))
    }

    fn combine(&self, inputs: &[ExportInput<'_>]) -> Option<Result<String, FormatError>> {
        Some(build_output(inputs))
    }

    fn rotation(&self) -> SpriteRotation {
        SpriteRotation::Unsupported
    }

    fn format_id(&self) -> &'static str {
        "godot"
    }

    fn file_extension(&self) -> &'static str {
        "tpsheet"
    }
}

fn build_output(inputs: &[ExportInput<'_>]) -> Result<String, FormatError> {
    let mut textures = Vec::with_capacity(inputs.len());
    for input in inputs {
        let mut sprites = Vec::with_capacity(input.atlas.frames.len());
        for frame in &input.atlas.frames {
            if frame.rotated {
                return Err(FormatError::Other(format!(
                    "Godot sprite sheets cannot contain rotated sprites: {}",
                    frame.id
                )));
            }
            let (w, h) = upright_size(frame);
            let src = frame.source_size;
            let sss = frame.sprite_source_size;
            sprites.push(Sprite {
                filename: image_name(&frame.id),
                region: Rect {
                    x: frame.frame.x as i64,
                    y: frame.frame.y as i64,
                    w: w as i64,
                    h: h as i64,
                },
                margin: Rect {
                    x: sss.x as i64,
                    y: sss.y as i64,
                    w: src.w as i64 - w as i64,
                    h: src.h as i64 - h as i64,
                },
            });
        }
        textures.push(Texture {
            image: &input.texture_filename,
            size: Size {
                w: input.atlas.size.w,
                h: input.atlas.size.h,
            },
            sprites,
        });
    }

    let (app, version, smartupdate) = match inputs.first() {
        Some(first) => smartupdate::meta_branding(first, "1.0"),
        None => ("FastPack", "1.0", None),
    };

    let output = Output {
        textures,
        meta: Meta {
            app,
            version,
            smartupdate,
        },
    };
    serde_json::to_string_pretty(&output).map_err(FormatError::Json)
}

#[derive(Serialize)]
struct Output<'a> {
    textures: Vec<Texture<'a>>,
    meta: Meta,
}

#[derive(Serialize)]
struct Texture<'a> {
    image: &'a str,
    size: Size,
    sprites: Vec<Sprite>,
}

#[derive(Serialize)]
struct Sprite {
    filename: String,
    region: Rect,
    margin: Rect,
}

#[derive(Serialize)]
struct Rect {
    x: i64,
    y: i64,
    w: i64,
    h: i64,
}

#[derive(Serialize)]
struct Size {
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct Meta {
    app: &'static str,
    version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    smartupdate: Option<String>,
}
