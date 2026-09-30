use std::collections::HashSet;
use std::fmt::Write as _;

use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter, SpriteRotation},
    smartupdate,
    text::upright_size,
};

/// Class shared by every sprite element; each sprite adds its own class.
const BASE_CLASS: &str = "sprite";

/// Exports a CSS stylesheet with one class per sprite.
///
/// Usage: `<span class="sprite hero"></span>`. Each sprite class sets the
/// texture, `width`/`height` and `background-position`. Sprite ids are turned
/// into valid class names (`ui/button` becomes `ui-button`) and made unique.
/// CSS backgrounds cannot be rotated, so rotation is disabled for this format.
/// Trimmed sprites are shown at their trimmed size. Multipack sheets are
/// combined into one stylesheet.
pub struct CssExporter;

impl Exporter for CssExporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        build_css(std::slice::from_ref(input))
    }

    fn combine(&self, inputs: &[ExportInput<'_>]) -> Option<Result<String, FormatError>> {
        Some(build_css(inputs))
    }

    fn rotation(&self) -> SpriteRotation {
        SpriteRotation::Unsupported
    }

    fn format_id(&self) -> &'static str {
        "css"
    }

    fn file_extension(&self) -> &'static str {
        "css"
    }
}

/// Turn a sprite id into a CSS class name.
///
/// Characters other than ASCII letters, digits, `-` and `_` become `-`, and a
/// leading `_` is added when the result would not be a valid identifier.
pub fn class_name(id: &str) -> String {
    let mut name: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let mut chars = name.chars();
    let needs_prefix = match (chars.next(), chars.next()) {
        (None, _) => true,
        (Some(c), _) if c.is_ascii_digit() => true,
        (Some('-'), None) => true,
        (Some('-'), Some(c)) => c.is_ascii_digit(),
        _ => false,
    };
    if needs_prefix {
        name.insert(0, '_');
    }
    name
}

/// Quote `s` as a CSS string.
fn css_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\a "),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn position(v: u32) -> String {
    if v == 0 {
        "0".to_string()
    } else {
        format!("-{v}px")
    }
}

fn build_css(inputs: &[ExportInput<'_>]) -> Result<String, FormatError> {
    let mut out = String::new();
    out.push_str("/*\n");
    match inputs.first() {
        Some(first) if first.hide_name => {
            let (app, _, smartupdate) = smartupdate::meta_branding(first, "1.0");
            let _ = writeln!(out, "   created with {app}");
            if let Some(hash) = smartupdate {
                let _ = writeln!(out, "   {hash}");
            }
        }
        _ => out.push_str("   created with FastPack\n"),
    }
    let _ = writeln!(
        out,
        "\n   usage: <span class=\"{BASE_CLASS} {{sprite-name}}\"></span>\n*/\n"
    );
    let _ = writeln!(
        out,
        ".{BASE_CLASS} {{display:inline-block; overflow:hidden; background-repeat:no-repeat;}}"
    );

    let mut used: HashSet<String> = HashSet::from([BASE_CLASS.to_string()]);
    for input in inputs {
        let image = css_string(&input.texture_filename);
        for frame in &input.atlas.frames {
            if frame.rotated {
                return Err(FormatError::Other(format!(
                    "CSS sprites cannot contain rotated sprites: {}",
                    frame.id
                )));
            }
            let base = class_name(&frame.id);
            let mut name = base.clone();
            let mut n = 2;
            while !used.insert(name.clone()) {
                name = format!("{base}-{n}");
                n += 1;
            }
            let (w, h) = upright_size(frame);
            let _ = writeln!(
                out,
                ".{name} {{background-image:url({image}); width:{w}px; height:{h}px; background-position:{} {};}}",
                position(frame.frame.x),
                position(frame.frame.y)
            );
        }
    }
    Ok(out)
}
