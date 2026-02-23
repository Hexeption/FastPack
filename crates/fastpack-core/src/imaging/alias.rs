use rustc_hash::FxHashMap;

use crate::types::sprite::Sprite;

/// Detect duplicate sprites by pixel content and mark them as aliases.
///
/// Sprites are grouped by `(width, height, content_hash)`. Within each group,
/// pixel bytes are compared exactly to confirm identity (handles hash collisions).
/// The first sprite encountered in each unique-pixel group becomes the canonical
/// entry; all others have `alias_of` set to that canonical sprite's id.
///
/// Returns `(unique, aliases)` where `unique` is suitable to pass to the packer
/// and `aliases` contains the stripped duplicates (each with `alias_of` set).
pub fn detect_aliases(sprites: Vec<Sprite>) -> (Vec<Sprite>, Vec<Sprite>) {
    // key → indices into `unique` that have this (w, h, hash).
    let mut buckets: FxHashMap<(u32, u32, u64), Vec<usize>> = FxHashMap::default();
    let mut unique: Vec<Sprite> = Vec::new();
    let mut aliases: Vec<Sprite> = Vec::new();

    for mut sprite in sprites {
        let w = sprite.image.width();
        let h = sprite.image.height();
        let key = (w, h, sprite.content_hash);

        let canon_id = buckets.get(&key).and_then(|indices| {
            indices.iter().find_map(|&i| {
                if pixels_equal(&sprite.image, &unique[i].image) {
                    Some(unique[i].id.clone())
                } else {
                    None
                }
            })
        });

        if let Some(id) = canon_id {
            sprite.alias_of = Some(id);
            aliases.push(sprite);
        } else {
            let idx = unique.len();
            buckets.entry(key).or_default().push(idx);
            unique.push(sprite);
        }
    }

    (unique, aliases)
}

fn pixels_equal(a: &image::DynamicImage, b: &image::DynamicImage) -> bool {
    match (a.as_rgba8(), b.as_rgba8()) {
        (Some(ra), Some(rb)) => ra.as_raw() == rb.as_raw(),
        _ => false,
    }
}
