//! Small helpers shared by the hand-written text and XML exporters.

use fastpack_core::types::atlas::AtlasFrame;

/// Escape `&`, `<`, `>`, `"` and `'` so `s` is safe in XML text and attributes.
pub(crate) fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Format a number without a trailing `.0` for whole values and never as `-0`.
pub(crate) fn fmt_num(v: f64) -> String {
    format!("{}", v + 0.0)
}

/// [`fmt_num`] for `f32` values, which keeps short decimals like `0.3` readable.
pub(crate) fn fmt_f32(v: f32) -> String {
    format!("{}", v + 0.0)
}

/// Width and height of the frame's pixels in their upright orientation.
///
/// `AtlasFrame::frame` stores the footprint in the texture, which has width and
/// height swapped when the sprite was rotated during packing.
pub(crate) fn upright_size(frame: &AtlasFrame) -> (u32, u32) {
    if frame.rotated {
        (frame.frame.h, frame.frame.w)
    } else {
        (frame.frame.w, frame.frame.h)
    }
}

/// Frame name with an image extension, as cocos2d, Sparrow and Godot expect.
///
/// Appends `.png` unless the name already ends in an image extension, which it
/// does when the project keeps source file extensions in sprite names.
pub(crate) fn image_name(id: &str) -> String {
    const IMAGE_EXTENSIONS: &[&str] = &[
        "png", "jpg", "jpeg", "webp", "bmp", "tga", "tif", "tiff", "gif",
    ];
    let has_ext = id
        .rsplit_once('.')
        .filter(|(stem, _)| !stem.is_empty() && !stem.ends_with('/'))
        .is_some_and(|(_, ext)| IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()));
    if has_ext {
        id.to_string()
    } else {
        format!("{id}.png")
    }
}

#[cfg(test)]
mod tests {
    use super::image_name;

    #[test]
    fn image_name_appends_png_only_when_missing() {
        assert_eq!(image_name("hero/run_01"), "hero/run_01.png");
        assert_eq!(image_name("hero/run_01.png"), "hero/run_01.png");
        assert_eq!(image_name("ui/Button.JPG"), "ui/Button.JPG");
        assert_eq!(image_name("v1.2"), "v1.2.png");
        assert_eq!(image_name("dir/.png"), "dir/.png.png");
    }
}
