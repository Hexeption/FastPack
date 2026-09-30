use std::collections::HashMap;
use std::fmt::Write as _;

use fastpack_core::types::atlas::AtlasFrame;

use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter},
    smartupdate,
    text::{fmt_f32, fmt_num, image_name, upright_size, xml_escape},
};

/// Exports atlas metadata as a Cocos2d-x property list (plist format 3).
///
/// Frame names get a `.png` suffix, matching TexturePacker's cocos2d-x output
/// (`SpriteFrameCache::getSpriteFrameByName("hero.png")`). Rotated frames are
/// stored 90° clockwise and `textureRect` carries the upright size, as
/// cocos2d-x expects. Alias frames are listed in their canonical frame's
/// `aliases` array when both live on the same sheet.
pub struct Cocos2dExporter;

impl Exporter for Cocos2dExporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        Ok(export_plist(input))
    }

    fn format_id(&self) -> &'static str {
        "cocos2d"
    }

    fn file_extension(&self) -> &'static str {
        "plist"
    }
}

fn frame_name(id: &str) -> String {
    image_name(id)
}

fn export_plist(input: &ExportInput<'_>) -> String {
    let atlas = input.atlas;

    // Aliases fold into their canonical frame's `aliases` array when the
    // canonical frame is on this sheet; otherwise they are written standalone.
    let ids: HashMap<&str, usize> = atlas
        .frames
        .iter()
        .enumerate()
        .filter(|(_, f)| f.alias_of.is_none())
        .map(|(i, f)| (f.id.as_str(), i))
        .collect();
    let mut aliases: HashMap<usize, Vec<&str>> = HashMap::new();
    let mut standalone: Vec<&AtlasFrame> = Vec::new();
    for frame in &atlas.frames {
        match frame.alias_of.as_deref().and_then(|c| ids.get(c)) {
            Some(&ci) => aliases.entry(ci).or_default().push(&frame.id),
            None => standalone.push(frame),
        }
    }

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(
        "<!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
    );
    out.push_str("<plist version=\"1.0\">\n");
    out.push_str("    <dict>\n");
    out.push_str("        <key>frames</key>\n");
    out.push_str("        <dict>\n");

    for frame in standalone {
        let index = ids.get(frame.id.as_str()).copied();
        let frame_aliases = index.and_then(|i| aliases.get(&i));
        write_frame(&mut out, frame, frame_aliases.map(Vec::as_slice));
    }

    out.push_str("        </dict>\n");
    out.push_str("        <key>metadata</key>\n");
    out.push_str("        <dict>\n");
    write_key(&mut out, 3, "format");
    out.push_str("            <integer>3</integer>\n");
    write_string(&mut out, 3, "pixelFormat", &input.pixel_format);
    write_string(&mut out, 3, "realTextureFileName", &input.texture_filename);
    write_string(
        &mut out,
        3,
        "size",
        &format!("{{{},{}}}", atlas.size.w, atlas.size.h),
    );
    let (_, _, smartupdate) = smartupdate::meta_branding(input, "3");
    if let Some(hash) = smartupdate {
        write_string(&mut out, 3, "smartupdate", &hash);
    }
    write_string(&mut out, 3, "textureFileName", &input.texture_filename);
    out.push_str("        </dict>\n");
    out.push_str("    </dict>\n");
    out.push_str("</plist>\n");
    out
}

fn write_frame(out: &mut String, frame: &AtlasFrame, aliases: Option<&[&str]>) {
    let (w, h) = upright_size(frame);
    let src = frame.source_size;
    let sss = frame.sprite_source_size;

    // Offset of the trimmed rect's centre from the source image's centre, y up.
    let offset_x = sss.x as f64 + sss.w as f64 / 2.0 - src.w as f64 / 2.0;
    let offset_y = src.h as f64 / 2.0 - (sss.y as f64 + sss.h as f64 / 2.0);

    write_key(out, 3, &frame_name(&frame.id));
    out.push_str("            <dict>\n");
    write_key(out, 4, "aliases");
    match aliases {
        Some(names) if !names.is_empty() => {
            out.push_str("                <array>\n");
            for name in names {
                let _ = writeln!(
                    out,
                    "                    <string>{}</string>",
                    xml_escape(&frame_name(name))
                );
            }
            out.push_str("                </array>\n");
        }
        _ => out.push_str("                <array/>\n"),
    }
    if let Some(p) = frame.pivot {
        let anchor = format!("{{{},{}}}", fmt_f32(p.x), fmt_f32(1.0 - p.y));
        write_string(out, 4, "anchor", &anchor);
    }
    write_string(
        out,
        4,
        "spriteOffset",
        &format!("{{{},{}}}", fmt_num(offset_x), fmt_num(offset_y)),
    );
    write_string(out, 4, "spriteSize", &format!("{{{w},{h}}}"));
    write_string(
        out,
        4,
        "spriteSourceSize",
        &format!("{{{},{}}}", src.w, src.h),
    );
    write_string(
        out,
        4,
        "textureRect",
        &format!("{{{{{},{}}},{{{w},{h}}}}}", frame.frame.x, frame.frame.y),
    );
    write_key(out, 4, "textureRotated");
    out.push_str(if frame.rotated {
        "                <true/>\n"
    } else {
        "                <false/>\n"
    });
    out.push_str("            </dict>\n");
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("    ");
    }
}

fn write_key(out: &mut String, level: usize, key: &str) {
    indent(out, level);
    let _ = writeln!(out, "<key>{}</key>", xml_escape(key));
}

fn write_string(out: &mut String, level: usize, key: &str, value: &str) {
    write_key(out, level, key);
    indent(out, level);
    let _ = writeln!(out, "<string>{}</string>", xml_escape(value));
}
