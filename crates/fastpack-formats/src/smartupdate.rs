//! TexturePacker-compatible SmartUpdate hash generation.
//!
//! Produces the `smartupdate` string embedded in the JSON `meta` block when
//! `hide_name` is enabled.  The format mirrors what TexturePacker emits:
//!
//! ```text
//! $TexturePacker:SmartUpdate:<content_hash>:<settings_hash>:<filelist_hash>$
//! ```
//!
//! Each hash is 32 lowercase hex characters (two concatenated 64-bit FNV-1a
//! rounds with different offsets), giving a 128-bit fingerprint per slot.

use crate::exporter::ExportInput;

const TEXTUREPACKER_APP: &str = "https://www.codeandweb.com/texturepacker";

const FNV_PRIME: u64 = 1099511628211;
const FNV_OFFSET_A: u64 = 14695981039346656037;
const FNV_OFFSET_B: u64 = 7160940396037360369;

/// Run two FNV-1a passes over `data` and return a 32-char lowercase hex string.
fn hash128(data: &[u8]) -> String {
    let mut ha = FNV_OFFSET_A;
    let mut hb = FNV_OFFSET_B;
    for &byte in data {
        ha ^= byte as u64;
        ha = ha.wrapping_mul(FNV_PRIME);
        hb ^= byte as u64;
        hb = hb.wrapping_mul(FNV_PRIME);
    }
    format!("{ha:016x}{hb:016x}")
}

/// Compute the TexturePacker SmartUpdate string for the given export input.
///
/// * **content_hash** — frame identifiers together with their packed rectangle
///   and source dimensions; changes whenever sprite pixel content changes.
/// * **settings_hash** — atlas size, scale factor, and pixel format; changes
///   whenever packer settings change.
/// * **filelist_hash** — sorted frame identifiers only; changes whenever the
///   set of packed sprites changes (added / removed / renamed).
pub fn compute(input: &ExportInput<'_>) -> String {
    let atlas = input.atlas;

    // Hash 1: content — ID + packed rect + source size for every frame.
    let mut content = String::new();
    for frame in &atlas.frames {
        content.push_str(&frame.id);
        content.push_str(&frame.frame.x.to_string());
        content.push_str(&frame.frame.y.to_string());
        content.push_str(&frame.frame.w.to_string());
        content.push_str(&frame.frame.h.to_string());
        content.push_str(&frame.source_size.w.to_string());
        content.push_str(&frame.source_size.h.to_string());
    }
    let hash1 = hash128(content.as_bytes());

    // Hash 2: settings — atlas dimensions, scale, and pixel format.
    let settings = format!(
        "{}{}{}{}",
        atlas.size.w, atlas.size.h, atlas.scale, input.pixel_format
    );
    let hash2 = hash128(settings.as_bytes());

    // Hash 3: file list — sorted frame IDs joined by newlines.
    let mut ids: Vec<&str> = atlas.frames.iter().map(|f| f.id.as_str()).collect();
    ids.sort_unstable();
    let filelist = ids.join("\n");
    let hash3 = hash128(filelist.as_bytes());

    format!("$TexturePacker:SmartUpdate:{hash1}:{hash2}:{hash3}$")
}

/// Return the `(app, version, smartupdate)` triple for an exporter's `meta` block.
///
/// With `hide_name` set this mimics TexturePacker; otherwise it reports FastPack
/// with `version` and omits the SmartUpdate hash.
pub(crate) fn meta_branding(
    input: &ExportInput<'_>,
    version: &'static str,
) -> (&'static str, &'static str, Option<String>) {
    if input.hide_name {
        (TEXTUREPACKER_APP, "3.0", Some(compute(input)))
    } else {
        ("FastPack", version, None)
    }
}
