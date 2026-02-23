use crate::{
    error::CoreError,
    types::{
        config::{LayoutConfig, MaxRectsHeuristic, SizeConstraint},
        rect::{Rect, Size},
        sprite::Sprite,
    },
};

use super::packer::{PackInput, PackOutput, PlacedSprite, Placement};

/// MaxRects rectangle bin-packing algorithm.
///
/// Maintains a list of maximal free rectangles. For each sprite the algorithm
/// scores every free rect under the configured heuristic, places the sprite in
/// the best-scoring position, then splits and prunes the free-rect list to
/// maintain the invariant that no free rect overlaps any placed sprite.
///
/// `ContactPointRule` falls back to `BestAreaFit` scoring until the full contact
/// computation is implemented in Phase 2.
#[derive(Default)]
pub struct MaxRects {
    pub heuristic: MaxRectsHeuristic,
}

impl super::packer::Packer for MaxRects {
    fn pack(&self, input: PackInput) -> Result<PackOutput, CoreError> {
        if input.sprites.is_empty() {
            return Err(CoreError::NoSprites);
        }
        Ok(pack_sprites(input, self.heuristic))
    }

    fn name(&self) -> &'static str {
        "maxrects"
    }
}

fn pack_sprites(input: PackInput, heuristic: MaxRectsHeuristic) -> PackOutput {
    let cfg = &input.config;
    let bp = cfg.border_padding;
    let sp = cfg.shape_padding;

    let canvas_w = cfg.max_width.saturating_sub(bp * 2);
    let canvas_h = cfg.max_height.saturating_sub(bp * 2);
    let mut free_rects: Vec<Rect> = vec![Rect::new(bp, bp, canvas_w, canvas_h)];

    let mut sprites = input.sprites;
    sprites.sort_unstable_by(|a, b| {
        let area_a = a.image.width() * a.image.height();
        let area_b = b.image.width() * b.image.height();
        area_b.cmp(&area_a)
    });

    let mut placed: Vec<PlacedSprite> = Vec::with_capacity(sprites.len());
    let mut overflow: Vec<Sprite> = Vec::new();

    for sprite in sprites {
        let sw = sprite.image.width();
        let sh = sprite.image.height();
        // Footprint includes shape_padding so gaps open between adjacent sprites.
        let fw = sw + sp;
        let fh = sh + sp;

        match find_best(&free_rects, fw, fh, cfg.allow_rotation, heuristic) {
            None => overflow.push(sprite),
            Some((dest_x, dest_y, rotated)) => {
                let (placed_w, placed_h) = if rotated { (sh, sw) } else { (sw, sh) };
                let (foot_w, foot_h) = if rotated {
                    (sh + sp, sw + sp)
                } else {
                    (fw, fh)
                };
                let dest = Rect::new(dest_x, dest_y, placed_w, placed_h);
                let footprint = Rect::new(dest_x, dest_y, foot_w, foot_h);
                split_and_prune(&mut free_rects, footprint);
                placed.push(PlacedSprite {
                    placement: Placement {
                        sprite_id: sprite.id.clone(),
                        dest,
                        rotated,
                    },
                    sprite,
                });
            }
        }
    }

    let atlas_size = compute_atlas_size(&placed, bp, cfg);
    PackOutput {
        placed,
        atlas_size,
        overflow,
    }
}

fn find_best(
    free_rects: &[Rect],
    fw: u32,
    fh: u32,
    allow_rotation: bool,
    heuristic: MaxRectsHeuristic,
) -> Option<(u32, u32, bool)> {
    let mut best_x = 0u32;
    let mut best_y = 0u32;
    let mut best_rotated = false;
    let mut best_primary = i64::MAX;
    let mut best_secondary = i64::MAX;
    let mut found = false;

    for r in free_rects {
        if fw <= r.w && fh <= r.h {
            let (p, s) = score(r, fw, fh, heuristic);
            if p < best_primary || (p == best_primary && s < best_secondary) {
                best_primary = p;
                best_secondary = s;
                best_x = r.x;
                best_y = r.y;
                best_rotated = false;
                found = true;
            }
        }
        if allow_rotation && fw != fh && fh <= r.w && fw <= r.h {
            let (p, s) = score(r, fh, fw, heuristic);
            if p < best_primary || (p == best_primary && s < best_secondary) {
                best_primary = p;
                best_secondary = s;
                best_x = r.x;
                best_y = r.y;
                best_rotated = true;
                found = true;
            }
        }
    }

    if found {
        Some((best_x, best_y, best_rotated))
    } else {
        None
    }
}

fn score(rect: &Rect, fw: u32, fh: u32, heuristic: MaxRectsHeuristic) -> (i64, i64) {
    let lw = (rect.w - fw) as i64;
    let lh = (rect.h - fh) as i64;
    match heuristic {
        MaxRectsHeuristic::BestShortSideFit => (lw.min(lh), lw.max(lh)),
        MaxRectsHeuristic::BestLongSideFit => (lw.max(lh), lw.min(lh)),
        MaxRectsHeuristic::BestAreaFit | MaxRectsHeuristic::ContactPointRule => {
            let waste = (rect.w * rect.h - fw * fh) as i64;
            (waste, lw.min(lh))
        }
        MaxRectsHeuristic::BottomLeftRule => (rect.y as i64, rect.x as i64),
    }
}

fn split_and_prune(free_rects: &mut Vec<Rect>, placed: Rect) {
    let mut splits: Vec<Rect> = Vec::new();
    let mut i = 0;

    while i < free_rects.len() {
        if free_rects[i].intersects(&placed) {
            let r = free_rects.remove(i);
            if placed.x > r.x {
                splits.push(Rect::new(r.x, r.y, placed.x - r.x, r.h));
            }
            if placed.right() < r.right() {
                splits.push(Rect::new(
                    placed.right(),
                    r.y,
                    r.right() - placed.right(),
                    r.h,
                ));
            }
            if placed.y > r.y {
                splits.push(Rect::new(r.x, r.y, r.w, placed.y - r.y));
            }
            if placed.bottom() < r.bottom() {
                splits.push(Rect::new(
                    r.x,
                    placed.bottom(),
                    r.w,
                    r.bottom() - placed.bottom(),
                ));
            }
        } else {
            i += 1;
        }
    }

    free_rects.extend(splits);
    prune(free_rects);
}

fn prune(rects: &mut Vec<Rect>) {
    let mut i = 0;
    while i < rects.len() {
        let mut dominated = false;
        for j in 0..rects.len() {
            if i != j && rects[j].contains(&rects[i]) {
                dominated = true;
                break;
            }
        }
        if dominated {
            rects.remove(i);
        } else {
            i += 1;
        }
    }
}

fn compute_atlas_size(placed: &[PlacedSprite], border_padding: u32, cfg: &LayoutConfig) -> Size {
    if placed.is_empty() {
        return Size { w: 0, h: 0 };
    }

    let mut max_x = 0u32;
    let mut max_y = 0u32;
    for ps in placed {
        max_x = max_x.max(ps.placement.dest.right());
        max_y = max_y.max(ps.placement.dest.bottom());
    }

    let w = constrain(max_x + border_padding, cfg.size_constraint);
    let h = constrain(max_y + border_padding, cfg.size_constraint);

    let (w, h) = if cfg.force_square {
        let side = w.max(h);
        (side, side)
    } else {
        (w, h)
    };

    Size {
        w: w.min(cfg.max_width),
        h: h.min(cfg.max_height),
    }
}

fn constrain(size: u32, constraint: SizeConstraint) -> u32 {
    match constraint {
        SizeConstraint::AnySize => size,
        SizeConstraint::Pot => next_pot(size),
        SizeConstraint::MultipleOf4 => round_up(size, 4),
        SizeConstraint::WordAligned => round_up(size, 2),
    }
}

fn next_pot(n: u32) -> u32 {
    if n <= 1 {
        return 1;
    }
    if n.is_power_of_two() {
        return n;
    }
    n.next_power_of_two()
}

fn round_up(n: u32, multiple: u32) -> u32 {
    if multiple <= 1 {
        return n;
    }
    let r = n % multiple;
    if r == 0 { n } else { n + (multiple - r) }
}
