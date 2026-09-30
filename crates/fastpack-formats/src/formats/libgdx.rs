use std::fmt::Write as _;

use fastpack_core::types::atlas::AtlasFrame;

use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter, SpriteRotation},
    text::upright_size,
};

/// Exports atlas metadata in the libGDX `TextureAtlas` text format.
///
/// Uses the current libGDX layout (`bounds:`, `offsets:`, `rotate:`), with
/// `split:` for nine-patch frames. Region names are sprite ids without a file
/// extension, as `TextureAtlas.findRegion()` expects. libGDX stores rotated
/// regions turned 90° counter-clockwise with `bounds` giving the upright size.
/// Multipack sheets are combined into one file with one page per texture.
pub struct LibGdxExporter;

impl Exporter for LibGdxExporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        write_atlas(std::slice::from_ref(input), &LIBGDX)
    }

    fn combine(&self, inputs: &[ExportInput<'_>]) -> Option<Result<String, FormatError>> {
        Some(write_atlas(inputs, &LIBGDX))
    }

    fn rotation(&self) -> SpriteRotation {
        SpriteRotation::CounterClockwise
    }

    fn format_id(&self) -> &'static str {
        "libgdx"
    }

    fn file_extension(&self) -> &'static str {
        "atlas"
    }
}

/// Punctuation differences between the libGDX and Spine flavours of the format.
pub(crate) struct AtlasStyle {
    /// Separator between a field name and its values.
    pub key_sep: &'static str,
    /// Separator between values of one field.
    pub value_sep: &'static str,
    /// Value written for `rotate` on rotated regions.
    pub rotate_value: &'static str,
    /// Name used in error messages.
    pub name: &'static str,
}

const LIBGDX: AtlasStyle = AtlasStyle {
    key_sep: ": ",
    value_sep: ", ",
    rotate_value: "true",
    name: "libGDX",
};

/// Map a FastPack pixel format name onto a libGDX `Pixmap.Format` name.
///
/// libGDX rejects unknown names, so formats it lacks fall back to the closest
/// format that can hold the decoded texture.
fn page_format(pixel_format: &str) -> &str {
    match pixel_format {
        "Alpha8" => "Alpha",
        "RGBA5551" => "RGBA8888",
        other => other,
    }
}

/// Write one page section per input, separated by blank lines.
pub(crate) fn write_atlas(
    inputs: &[ExportInput<'_>],
    style: &AtlasStyle,
) -> Result<String, FormatError> {
    let mut out = String::new();
    for (i, input) in inputs.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        write_page(&mut out, input, style)?;
    }
    Ok(out)
}

fn check_name(name: &str, style: &AtlasStyle) -> Result<(), FormatError> {
    if name.is_empty() || name.contains([':', '\n', '\r']) || name.trim() != name {
        return Err(FormatError::Other(format!(
            "{} atlas names cannot be empty, contain ':' or line breaks, or start or end with whitespace: {name:?}",
            style.name
        )));
    }
    Ok(())
}

fn write_page(
    out: &mut String,
    input: &ExportInput<'_>,
    style: &AtlasStyle,
) -> Result<(), FormatError> {
    let AtlasStyle {
        key_sep: k,
        value_sep: v,
        ..
    } = style;
    let atlas = input.atlas;

    check_name(&input.texture_filename, style)?;
    let _ = writeln!(out, "{}", input.texture_filename);
    let _ = writeln!(out, "size{k}{}{v}{}", atlas.size.w, atlas.size.h);
    let _ = writeln!(out, "format{k}{}", page_format(&input.pixel_format));
    let _ = writeln!(out, "filter{k}Linear{v}Linear");
    let _ = writeln!(out, "repeat{k}none");

    for frame in &atlas.frames {
        check_name(&frame.id, style)?;
        write_region(out, frame, style);
    }
    Ok(())
}

fn write_region(out: &mut String, frame: &AtlasFrame, style: &AtlasStyle) {
    let AtlasStyle {
        key_sep: k,
        value_sep: v,
        rotate_value,
        ..
    } = style;
    let (w, h) = upright_size(frame);
    let src = frame.source_size;
    let sss = frame.sprite_source_size;

    let _ = writeln!(out, "{}", frame.id);
    let _ = writeln!(
        out,
        "bounds{k}{}{v}{}{v}{w}{v}{h}",
        frame.frame.x, frame.frame.y
    );

    // Whitespace stripped from the left and bottom edges, plus the original size.
    let offset_x = sss.x;
    let offset_y = src.h as i64 - (sss.y as i64 + h as i64);
    if offset_x != 0 || offset_y != 0 || w != src.w || h != src.h {
        let _ = writeln!(
            out,
            "offsets{k}{offset_x}{v}{offset_y}{v}{}{v}{}",
            src.w, src.h
        );
    }

    if frame.rotated {
        let _ = writeln!(out, "rotate{k}{rotate_value}");
    }

    if let Some(np) = frame.nine_patch {
        // Nine-patch borders are measured on the source image; shift them onto
        // the trimmed region that is actually stored in the texture.
        let trim_left = sss.x.max(0) as u32;
        let trim_top = sss.y.max(0) as u32;
        let trim_right = (src.w as i64 - sss.x as i64 - w as i64).max(0) as u32;
        let trim_bottom = (src.h as i64 - sss.y as i64 - h as i64).max(0) as u32;
        let _ = writeln!(
            out,
            "split{k}{}{v}{}{v}{}{v}{}",
            np.left.saturating_sub(trim_left),
            np.right.saturating_sub(trim_right),
            np.top.saturating_sub(trim_top),
            np.bottom.saturating_sub(trim_bottom)
        );
    }
}
