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
