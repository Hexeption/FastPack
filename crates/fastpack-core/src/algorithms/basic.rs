use crate::{
    error::CoreError,
    types::rect::{Rect, Size},
};

use super::packer::{PackInput, PackOutput, Packer, PlacedSprite, Placement};

/// Basic strip packing algorithm.
///
/// Sprites are sorted by descending height and placed into rows left-to-right.
/// A new row starts when the next sprite does not fit in the remaining width of
/// the current row. Each row's height equals the tallest sprite placed in it.
///
/// This algorithm is fast with predictable output but produces larger atlases
/// than MaxRects. It does not attempt rotation.
#[derive(Default)]
pub struct Basic;

impl Packer for Basic {
    fn pack(&self, input: PackInput) -> Result<PackOutput, CoreError> {
        if input.sprites.is_empty() {
            return Err(CoreError::NoSprites);
        }
        Ok(pack_basic(input))
    }

    fn name(&self) -> &'static str {
        "basic"
    }
}

fn pack_basic(input: PackInput) -> PackOutput {
    let cfg = &input.config;
    let bp = cfg.border_padding;
    let sp = cfg.shape_padding;

    let mut sprites = input.sprites;
    // Tallest-first minimises wasted space at row ends.
    sprites.sort_unstable_by(|a, b| b.image.height().cmp(&a.image.height()));

    let max_canvas_w = cfg.max_width.saturating_sub(bp * 2);
    let max_canvas_h = cfg.max_height.saturating_sub(bp * 2);

    let mut placed = Vec::with_capacity(sprites.len());
    let mut overflow = Vec::new();

    let mut cursor_x: u32 = 0;
    let mut cursor_y: u32 = 0;
    let mut row_h: u32 = 0;
    let mut atlas_w: u32 = 0;
    let mut atlas_h: u32 = 0;

    for sprite in sprites {
        let sw = sprite.image.width();
        let sh = sprite.image.height();
        let fw = sw + sp;
        let fh = sh + sp;

        // Wrap to a new row when the sprite doesn't fit horizontally.
        if cursor_x > 0 && cursor_x + fw > max_canvas_w {
            cursor_y += row_h;
            cursor_x = 0;
            row_h = 0;
        }

        // Vertical overflow.
        if cursor_y + sh > max_canvas_h {
            overflow.push(sprite);
            continue;
        }

        let dest_x = bp + cursor_x;
        let dest_y = bp + cursor_y;

        atlas_w = atlas_w.max(cursor_x + sw);
        atlas_h = atlas_h.max(cursor_y + sh);
        row_h = row_h.max(fh);

        placed.push(PlacedSprite {
            placement: Placement {
                sprite_id: sprite.id.clone(),
                dest: Rect::new(dest_x, dest_y, sw, sh),
                rotated: false,
            },
            sprite,
        });

        cursor_x += fw;
    }

    let raw_w = if placed.is_empty() {
        0
    } else {
        atlas_w + bp * 2
    };
    let raw_h = if placed.is_empty() {
        0
    } else {
        atlas_h + bp * 2
    };

    let mut w = cfg.size_constraint.apply(raw_w).min(cfg.max_width);
    let mut h = cfg.size_constraint.apply(raw_h).min(cfg.max_height);

    if cfg.force_square {
        let side = w.max(h);
        w = side;
        h = side;
    }

    PackOutput {
        placed,
        atlas_size: Size { w, h },
        overflow,
    }
}
