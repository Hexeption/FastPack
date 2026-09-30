use std::fmt::Write as _;

use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter},
    smartupdate,
    text::{fmt_f32, image_name, upright_size, xml_escape},
};

/// Exports atlas metadata in Starling / Sparrow `TextureAtlas` XML format.
///
/// Each frame is a `SubTexture` element. Sprite names get a `.png` suffix,
/// matching TexturePacker's Sparrow output. `x`/`y`/`width`/`height` describe
/// the region as stored in the texture (rotated sprites keep their rotated
/// footprint, turned 90° clockwise as Starling expects), while
/// `frameX`/`frameY`/`frameWidth`/`frameHeight` describe the untrimmed sprite
/// and are only written when the sprite was trimmed.
pub struct SparrowExporter;

impl Exporter for SparrowExporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        Ok(export_xml(input))
    }

    fn format_id(&self) -> &'static str {
        "sparrow"
    }

    fn file_extension(&self) -> &'static str {
        "xml"
    }
}

fn export_xml(input: &ExportInput<'_>) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let (app, _, smartupdate) = smartupdate::meta_branding(input, "1.0");
    if input.hide_name {
        let _ = writeln!(out, "<!-- Created with TexturePacker {app} -->");
    } else {
        let _ = writeln!(out, "<!-- Created with {app} -->");
    }
    if let Some(hash) = smartupdate {
        let _ = writeln!(out, "<!-- {hash} -->");
    }
    let _ = writeln!(
        out,
        "<TextureAtlas imagePath=\"{}\">",
        xml_escape(&input.texture_filename)
    );

    for frame in &input.atlas.frames {
        let r = frame.frame;
        let _ = write!(
            out,
            "    <SubTexture name=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
            xml_escape(&image_name(&frame.id)),
            r.x,
            r.y,
            r.w,
            r.h
        );

        let (w, h) = upright_size(frame);
        let src = frame.source_size;
        let sss = frame.sprite_source_size;
        if sss.x != 0 || sss.y != 0 || w != src.w || h != src.h {
            let _ = write!(
                out,
                " frameX=\"{}\" frameY=\"{}\" frameWidth=\"{}\" frameHeight=\"{}\"",
                -sss.x, -sss.y, src.w, src.h
            );
        }
        if frame.rotated {
            out.push_str(" rotated=\"true\"");
        }
        if let Some(p) = frame.pivot {
            let _ = write!(
                out,
                " pivotX=\"{}\" pivotY=\"{}\"",
                fmt_f32(p.x * src.w as f32),
                fmt_f32(p.y * src.h as f32)
            );
        }
        out.push_str("/>\n");
    }

    out.push_str("</TextureAtlas>\n");
    out
}
