use crate::{
    error::CoreError,
    types::rect::{Rect, Size},
};

use super::packer::{PackInput, PackOutput, Packer, PlacedSprite, Placement};

/// Grid packing algorithm.
///
/// Every sprite occupies an identically sized cell. Sprites are placed
/// left-to-right then top-to-bottom. Cell dimensions default to the maximum
/// sprite width and height across all input sprites; they can be overridden via
/// `cell_width` / `cell_height` for fixed-cell sprite sheets.
///
/// Sprites that are larger than the cell are sent to `overflow` without
/// cropping. The atlas width is determined by the number of columns that fit
/// within `max_width`; the height grows with each row added.
#[derive(Default)]
pub struct Grid {
    /// Fixed cell width in pixels. `None` → widest sprite.
    pub cell_width: Option<u32>,
    /// Fixed cell height in pixels. `None` → tallest sprite.
    pub cell_height: Option<u32>,
}

impl Packer for Grid {
    fn pack(&self, input: PackInput) -> Result<PackOutput, CoreError> {
        if input.sprites.is_empty() {
            return Err(CoreError::NoSprites);
        }
        Ok(pack_grid(input, self.cell_width, self.cell_height))
    }

    fn name(&self) -> &'static str {
        "grid"
    }
}

fn pack_grid(input: PackInput, fixed_cw: Option<u32>, fixed_ch: Option<u32>) -> PackOutput {
    let cfg = &input.config;
    let bp = cfg.border_padding;
    let sp = cfg.shape_padding;

    let cell_w = fixed_cw.unwrap_or_else(|| {
        input
            .sprites
            .iter()
            .map(|s| s.image.width())
            .max()
            .unwrap_or(1)
    });
    let cell_h = fixed_ch.unwrap_or_else(|| {
        input
            .sprites
            .iter()
            .map(|s| s.image.height())
            .max()
            .unwrap_or(1)
    });

    let step_x = cell_w + sp;
    let step_y = cell_h + sp;

    // How many columns fit in the usable canvas width.
    let usable_w = cfg.max_width.saturating_sub(bp * 2);
    let cols = ((usable_w + sp) / step_x).max(1);

    let mut placed = Vec::with_capacity(input.sprites.len());
    let mut overflow = Vec::new();
    let mut max_row = 0u32;

    for (idx, sprite) in input.sprites.into_iter().enumerate() {
        let col = idx as u32 % cols;
        let row = idx as u32 / cols;
        let x = bp + col * step_x;
        let y = bp + row * step_y;

        // Vertical overflow check.
        if y + cell_h + bp > cfg.max_height {
            overflow.push(sprite);
            continue;
        }

        // Center the sprite within its cell.
        let sw = sprite.image.width();
        let sh = sprite.image.height();
        let dest_x = x + (cell_w.saturating_sub(sw)) / 2;
        let dest_y = y + (cell_h.saturating_sub(sh)) / 2;

        max_row = max_row.max(row);

        placed.push(PlacedSprite {
            placement: Placement {
                sprite_id: sprite.id.clone(),
                dest: Rect::new(dest_x, dest_y, sw, sh),
                rotated: false,
            },
            sprite,
        });
    }

    let row_count = if placed.is_empty() { 0 } else { max_row + 1 };

    // Atlas covers exactly the occupied columns × rows, with padding.
    let raw_w = if row_count == 0 {
        0
    } else {
        bp + cols * step_x - sp + bp
    };
    let raw_h = if row_count == 0 {
        0
    } else {
        bp + row_count * step_y - sp + bp
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
