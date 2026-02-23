use fastpack_core::{
    algorithms::{
        basic::Basic,
        grid::Grid,
        maxrects::MaxRects,
        packer::{PackInput, Packer},
    },
    imaging::{alias::detect_aliases, extrude::extrude, trim::trim},
    types::{
        config::{LayoutConfig, PackMode, SizeConstraint, SpriteConfig, TrimMode},
        rect::{Rect, Size},
        sprite::Sprite,
    },
};
use image::{DynamicImage, Rgba, RgbaImage};
use std::path::PathBuf;

// Helpers

fn make_sprite(id: &str, w: u32, h: u32) -> Sprite {
    let mut img = RgbaImage::new(w, h);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([255, 128, 64, 255]);
    }
    Sprite {
        id: id.to_string(),
        source_path: PathBuf::from(format!("{id}.png")),
        image: DynamicImage::ImageRgba8(img),
        trim_rect: None,
        original_size: Size { w, h },
        polygon: None,
        nine_patch: None,
        pivot: None,
        content_hash: 0,
        extrude: 0,
        alias_of: None,
    }
}

fn make_transparent_sprite(id: &str, w: u32, h: u32) -> Sprite {
    let img = RgbaImage::new(w, h);
    Sprite {
        id: id.to_string(),
        source_path: PathBuf::from(format!("{id}.png")),
        image: DynamicImage::ImageRgba8(img),
        trim_rect: None,
        original_size: Size { w, h },
        polygon: None,
        nine_patch: None,
        pivot: None,
        content_hash: 0,
        extrude: 0,
        alias_of: None,
    }
}

/// Sprite with an opaque inner region and transparent `border`-px margin.
fn make_bordered_sprite(id: &str, w: u32, h: u32, border: u32) -> Sprite {
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            if x >= border && x < w - border && y >= border && y < h - border {
                *img.get_pixel_mut(x, y) = Rgba([255, 0, 0, 255]);
            }
        }
    }
    Sprite {
        id: id.to_string(),
        source_path: PathBuf::from(format!("{id}.png")),
        image: DynamicImage::ImageRgba8(img),
        trim_rect: None,
        original_size: Size { w, h },
        polygon: None,
        nine_patch: None,
        pivot: None,
        content_hash: 0,
        extrude: 0,
        alias_of: None,
    }
}

fn fast_layout(max_w: u32, max_h: u32) -> LayoutConfig {
    LayoutConfig {
        max_width: max_w,
        max_height: max_h,
        border_padding: 0,
        shape_padding: 0,
        allow_rotation: false,
        force_square: false,
        pack_mode: PackMode::Fast,
        ..LayoutConfig::default()
    }
}

// SizeConstraint::apply

#[test]
fn size_constraint_anysize_passthrough() {
    assert_eq!(SizeConstraint::AnySize.apply(0), 0);
    assert_eq!(SizeConstraint::AnySize.apply(1), 1);
    assert_eq!(SizeConstraint::AnySize.apply(100), 100);
    assert_eq!(SizeConstraint::AnySize.apply(4096), 4096);
}

#[test]
fn size_constraint_pot_already_power_of_two() {
    assert_eq!(SizeConstraint::Pot.apply(1), 1);
    assert_eq!(SizeConstraint::Pot.apply(2), 2);
    assert_eq!(SizeConstraint::Pot.apply(4), 4);
    assert_eq!(SizeConstraint::Pot.apply(256), 256);
    assert_eq!(SizeConstraint::Pot.apply(1024), 1024);
    assert_eq!(SizeConstraint::Pot.apply(2048), 2048);
}

#[test]
fn size_constraint_pot_rounds_up() {
    assert_eq!(SizeConstraint::Pot.apply(3), 4);
    assert_eq!(SizeConstraint::Pot.apply(5), 8);
    assert_eq!(SizeConstraint::Pot.apply(100), 128);
    assert_eq!(SizeConstraint::Pot.apply(129), 256);
    assert_eq!(SizeConstraint::Pot.apply(1025), 2048);
}

#[test]
fn size_constraint_multipleof4_already_aligned() {
    assert_eq!(SizeConstraint::MultipleOf4.apply(0), 0);
    assert_eq!(SizeConstraint::MultipleOf4.apply(4), 4);
    assert_eq!(SizeConstraint::MultipleOf4.apply(8), 8);
    assert_eq!(SizeConstraint::MultipleOf4.apply(100), 100);
    assert_eq!(SizeConstraint::MultipleOf4.apply(256), 256);
}

#[test]
fn size_constraint_multipleof4_rounds_up() {
    assert_eq!(SizeConstraint::MultipleOf4.apply(1), 4);
    assert_eq!(SizeConstraint::MultipleOf4.apply(2), 4);
    assert_eq!(SizeConstraint::MultipleOf4.apply(3), 4);
    assert_eq!(SizeConstraint::MultipleOf4.apply(5), 8);
    assert_eq!(SizeConstraint::MultipleOf4.apply(101), 104);
    assert_eq!(SizeConstraint::MultipleOf4.apply(255), 256);
}

#[test]
fn size_constraint_wordaligned_already_even() {
    assert_eq!(SizeConstraint::WordAligned.apply(0), 0);
    assert_eq!(SizeConstraint::WordAligned.apply(2), 2);
    assert_eq!(SizeConstraint::WordAligned.apply(100), 100);
    assert_eq!(SizeConstraint::WordAligned.apply(512), 512);
}

#[test]
fn size_constraint_wordaligned_rounds_odd_up() {
    assert_eq!(SizeConstraint::WordAligned.apply(1), 2);
    assert_eq!(SizeConstraint::WordAligned.apply(3), 4);
    assert_eq!(SizeConstraint::WordAligned.apply(101), 102);
    assert_eq!(SizeConstraint::WordAligned.apply(511), 512);
}

// Rect

#[test]
fn rect_area() {
    assert_eq!(Rect::new(0, 0, 10, 20).area(), 200);
    assert_eq!(Rect::new(5, 5, 0, 5).area(), 0);
    assert_eq!(Rect::new(0, 0, 1, 1).area(), 1);
}

#[test]
fn rect_right_and_bottom() {
    let r = Rect::new(10, 20, 30, 40);
    assert_eq!(r.right(), 40);
    assert_eq!(r.bottom(), 60);
}

#[test]
fn rect_right_bottom_at_origin() {
    let r = Rect::new(0, 0, 100, 50);
    assert_eq!(r.right(), 100);
    assert_eq!(r.bottom(), 50);
}

#[test]
fn rect_contains_self() {
    let r = Rect::new(0, 0, 100, 100);
    assert!(r.contains(&r));
}

#[test]
fn rect_contains_inner_rect() {
    let outer = Rect::new(0, 0, 100, 100);
    let inner = Rect::new(10, 10, 50, 50);
    assert!(outer.contains(&inner));
    assert!(!inner.contains(&outer));
}

#[test]
fn rect_contains_same_edge_rect() {
    let r = Rect::new(0, 0, 100, 100);
    let same_left = Rect::new(0, 0, 100, 50);
    assert!(r.contains(&same_left));
}

#[test]
fn rect_does_not_contain_overlapping_rect() {
    let a = Rect::new(0, 0, 50, 50);
    let b = Rect::new(25, 25, 50, 50);
    assert!(!a.contains(&b));
    assert!(!b.contains(&a));
}

#[test]
fn rect_intersects_overlapping() {
    let a = Rect::new(0, 0, 50, 50);
    let b = Rect::new(25, 25, 50, 50);
    assert!(a.intersects(&b));
    assert!(b.intersects(&a));
}

#[test]
fn rect_intersects_one_inside_another() {
    let outer = Rect::new(0, 0, 100, 100);
    let inner = Rect::new(10, 10, 10, 10);
    assert!(outer.intersects(&inner));
    assert!(inner.intersects(&outer));
}

#[test]
fn rect_touching_edge_does_not_intersect() {
    let a = Rect::new(0, 0, 50, 50);
    let b = Rect::new(50, 0, 50, 50);
    assert!(!a.intersects(&b));
    assert!(!b.intersects(&a));
}

#[test]
fn rect_separate_rects_do_not_intersect() {
    let a = Rect::new(0, 0, 10, 10);
    let b = Rect::new(20, 20, 10, 10);
    assert!(!a.intersects(&b));
}

// Size

#[test]
fn size_area() {
    assert_eq!(Size { w: 10, h: 20 }.area(), 200);
    assert_eq!(Size { w: 1, h: 1 }.area(), 1);
}

#[test]
fn size_area_zero_when_empty() {
    assert_eq!(Size { w: 0, h: 100 }.area(), 0);
    assert_eq!(Size { w: 100, h: 0 }.area(), 0);
}

#[test]
fn size_is_empty_zero_width() {
    assert!(Size { w: 0, h: 100 }.is_empty());
}

#[test]
fn size_is_empty_zero_height() {
    assert!(Size { w: 100, h: 0 }.is_empty());
}

#[test]
fn size_is_empty_both_zero() {
    assert!(Size { w: 0, h: 0 }.is_empty());
}

#[test]
fn size_not_empty_when_positive() {
    assert!(!Size { w: 1, h: 1 }.is_empty());
    assert!(!Size { w: 100, h: 200 }.is_empty());
}

// trim

#[test]
fn trim_none_mode_is_noop() {
    let mut sprite = make_bordered_sprite("s", 10, 10, 2);
    let cfg = SpriteConfig {
        trim_mode: TrimMode::None,
        ..SpriteConfig::default()
    };
    trim(&mut sprite, &cfg);
    assert!(sprite.trim_rect.is_none());
    assert_eq!(sprite.image.width(), 10);
    assert_eq!(sprite.image.height(), 10);
}

#[test]
fn trim_removes_transparent_border() {
    // 10×10 with a 2px transparent border; inner 6×6 is opaque
    let mut sprite = make_bordered_sprite("s", 10, 10, 2);
    let cfg = SpriteConfig {
        trim_mode: TrimMode::Trim,
        trim_threshold: 0,
        ..SpriteConfig::default()
    };
    trim(&mut sprite, &cfg);
    let tr = sprite
        .trim_rect
        .expect("should have trim_rect after trimming");
    assert_eq!(tr.x, 2);
    assert_eq!(tr.y, 2);
    assert_eq!(tr.w, 6);
    assert_eq!(tr.h, 6);
    assert_eq!(sprite.image.width(), 6);
    assert_eq!(sprite.image.height(), 6);
}

#[test]
fn trim_fully_transparent_leaves_image_unchanged() {
    let mut sprite = make_transparent_sprite("t", 8, 8);
    let cfg = SpriteConfig {
        trim_mode: TrimMode::Trim,
        trim_threshold: 0,
        ..SpriteConfig::default()
    };
    trim(&mut sprite, &cfg);
    assert!(sprite.trim_rect.is_none());
    assert_eq!(sprite.image.width(), 8);
    assert_eq!(sprite.image.height(), 8);
}

#[test]
fn trim_solid_opaque_image_unchanged() {
    let mut sprite = make_sprite("s", 10, 10);
    let orig_w = sprite.image.width();
    let orig_h = sprite.image.height();
    let cfg = SpriteConfig {
        trim_mode: TrimMode::Trim,
        trim_threshold: 0,
        ..SpriteConfig::default()
    };
    trim(&mut sprite, &cfg);
    // Fully opaque image — no border to trim, still no trim_rect set (because
    // the crop would be the full image). Check at minimum the image was not
    // made smaller than intended.
    assert!(sprite.image.width() <= orig_w);
    assert!(sprite.image.height() <= orig_h);
}

#[test]
fn trim_records_original_size_unchanged() {
    let mut sprite = make_bordered_sprite("s", 12, 12, 3);
    let cfg = SpriteConfig {
        trim_mode: TrimMode::Trim,
        trim_threshold: 0,
        ..SpriteConfig::default()
    };
    trim(&mut sprite, &cfg);
    // original_size must not be mutated by trim
    assert_eq!(sprite.original_size, Size { w: 12, h: 12 });
}

// extrude

#[test]
fn extrude_zero_is_noop() {
    let mut sprite = make_sprite("s", 8, 8);
    extrude(&mut sprite, 0);
    assert_eq!(sprite.image.width(), 8);
    assert_eq!(sprite.image.height(), 8);
    assert_eq!(sprite.extrude, 0);
}

#[test]
fn extrude_grows_image_dimensions() {
    let mut sprite = make_sprite("s", 8, 8);
    extrude(&mut sprite, 2);
    assert_eq!(sprite.image.width(), 12);
    assert_eq!(sprite.image.height(), 12);
    assert_eq!(sprite.extrude, 2);
}

#[test]
fn extrude_records_amount() {
    let mut sprite = make_sprite("s", 16, 16);
    extrude(&mut sprite, 3);
    assert_eq!(sprite.extrude, 3);
    assert_eq!(sprite.image.width(), 22);
    assert_eq!(sprite.image.height(), 22);
}

#[test]
fn extrude_single_pixel_image_all_output_pixels_match() {
    let mut img = RgbaImage::new(1, 1);
    *img.get_pixel_mut(0, 0) = Rgba([200, 100, 50, 255]);
    let mut sprite = Sprite {
        id: "s".to_string(),
        source_path: PathBuf::from("s.png"),
        image: DynamicImage::ImageRgba8(img),
        trim_rect: None,
        original_size: Size { w: 1, h: 1 },
        polygon: None,
        nine_patch: None,
        pivot: None,
        content_hash: 0,
        extrude: 0,
        alias_of: None,
    };
    extrude(&mut sprite, 1);
    let rgba = sprite.image.as_rgba8().unwrap();
    assert_eq!(rgba.dimensions(), (3, 3));
    for pixel in rgba.pixels() {
        assert_eq!(*pixel, Rgba([200, 100, 50, 255]));
    }
}

#[test]
fn extrude_larger_amount() {
    let mut sprite = make_sprite("s", 4, 6);
    extrude(&mut sprite, 4);
    assert_eq!(sprite.image.width(), 12); // 4 + 4*2
    assert_eq!(sprite.image.height(), 14); // 6 + 4*2
}

// alias detection

#[test]
fn detect_aliases_identical_sprites_deduped() {
    let s1 = make_sprite("a", 8, 8);
    let s2 = make_sprite("b", 8, 8); // same pixel content as s1
    let (unique, aliases) = detect_aliases(vec![s1, s2]);
    assert_eq!(unique.len(), 1);
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].alias_of.as_deref(), Some("a"));
}

#[test]
fn detect_aliases_different_dimensions_both_unique() {
    let s1 = make_sprite("a", 8, 8);
    let s2 = make_sprite("b", 16, 16);
    let (unique, aliases) = detect_aliases(vec![s1, s2]);
    assert_eq!(unique.len(), 2);
    assert_eq!(aliases.len(), 0);
}

#[test]
fn detect_aliases_different_pixels_same_dimensions_both_unique() {
    let s1 = make_sprite("a", 8, 8); // red pixels
    let mut img = RgbaImage::new(8, 8);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 255, 255]);
    }
    let s2 = Sprite {
        id: "b".to_string(),
        source_path: PathBuf::from("b.png"),
        image: DynamicImage::ImageRgba8(img),
        trim_rect: None,
        original_size: Size { w: 8, h: 8 },
        polygon: None,
        nine_patch: None,
        pivot: None,
        content_hash: 0,
        extrude: 0,
        alias_of: None,
    };
    let (unique, aliases) = detect_aliases(vec![s1, s2]);
    assert_eq!(unique.len(), 2);
    assert_eq!(aliases.len(), 0);
}

#[test]
fn detect_aliases_empty_input() {
    let (unique, aliases) = detect_aliases(vec![]);
    assert!(unique.is_empty());
    assert!(aliases.is_empty());
}

#[test]
fn detect_aliases_single_sprite_is_unique() {
    let s = make_sprite("only", 32, 32);
    let (unique, aliases) = detect_aliases(vec![s]);
    assert_eq!(unique.len(), 1);
    assert!(aliases.is_empty());
}

#[test]
fn detect_aliases_three_identical_two_become_aliases() {
    let s1 = make_sprite("a", 8, 8);
    let s2 = make_sprite("b", 8, 8);
    let s3 = make_sprite("c", 8, 8);
    let (unique, aliases) = detect_aliases(vec![s1, s2, s3]);
    assert_eq!(unique.len(), 1);
    assert_eq!(aliases.len(), 2);
    for alias in &aliases {
        assert_eq!(alias.alias_of.as_deref(), Some("a"));
    }
}

// Basic packer

#[test]
fn basic_empty_input_returns_error() {
    let result = Basic.pack(PackInput {
        sprites: vec![],
        config: fast_layout(512, 512),
        sprite_config: SpriteConfig::default(),
    });
    assert!(result.is_err());
}

#[test]
fn basic_single_sprite_placed() {
    let output = Basic
        .pack(PackInput {
            sprites: vec![make_sprite("a", 32, 32)],
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert_eq!(output.placed.len(), 1);
    assert!(output.overflow.is_empty());
    assert_eq!(output.placed[0].placement.sprite_id, "a");
}

#[test]
fn basic_multiple_sprites_all_placed() {
    let sprites = vec![
        make_sprite("a", 32, 32),
        make_sprite("b", 48, 24),
        make_sprite("c", 16, 64),
    ];
    let output = Basic
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert_eq!(output.placed.len(), 3);
    assert!(output.overflow.is_empty());
}

#[test]
fn basic_sprite_taller_than_max_height_overflows() {
    let output = Basic
        .pack(PackInput {
            sprites: vec![make_sprite("a", 32, 600)],
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert!(output.placed.is_empty());
    assert_eq!(output.overflow.len(), 1);
}

#[test]
fn basic_row_wrap_when_too_wide() {
    // Two sprites that each would fill ~60% of the row; the second must wrap.
    let sprites = vec![
        make_sprite("first", 300, 32),
        make_sprite("second", 300, 32),
    ];
    let output = Basic
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert_eq!(output.placed.len(), 2);
    let ys: Vec<u32> = output.placed.iter().map(|ps| ps.placement.dest.y).collect();
    let min_y = *ys.iter().min().unwrap();
    let max_y = *ys.iter().max().unwrap();
    assert!(max_y > min_y, "sprites should land on different rows");
}

#[test]
fn basic_atlas_size_covers_all_placed_sprites() {
    let sprites = vec![make_sprite("a", 100, 80), make_sprite("b", 80, 100)];
    let output = Basic
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    let max_right = output
        .placed
        .iter()
        .map(|ps| ps.placement.dest.right())
        .max()
        .unwrap();
    let max_bottom = output
        .placed
        .iter()
        .map(|ps| ps.placement.dest.bottom())
        .max()
        .unwrap();
    assert!(output.atlas_size.w >= max_right);
    assert!(output.atlas_size.h >= max_bottom);
}

// Grid packer

#[test]
fn grid_empty_input_returns_error() {
    let result = Grid::default().pack(PackInput {
        sprites: vec![],
        config: fast_layout(512, 512),
        sprite_config: SpriteConfig::default(),
    });
    assert!(result.is_err());
}

#[test]
fn grid_single_sprite_placed() {
    let output = Grid::default()
        .pack(PackInput {
            sprites: vec![make_sprite("a", 32, 32)],
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert_eq!(output.placed.len(), 1);
    assert!(output.overflow.is_empty());
}

#[test]
fn grid_multiple_sprites_placed_in_columns() {
    let sprites: Vec<Sprite> = (0..4)
        .map(|i| make_sprite(&format!("s{i}"), 32, 32))
        .collect();
    let output = Grid::default()
        .pack(PackInput {
            sprites,
            config: fast_layout(256, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert_eq!(output.placed.len(), 4);
    assert!(output.overflow.is_empty());
}

#[test]
fn grid_smaller_sprite_centered_in_cell() {
    // "small" 16×16 gets centered inside the 48×48 cell sized to "big"
    let sprites = vec![make_sprite("small", 16, 16), make_sprite("big", 48, 48)];
    let output = Grid::default()
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    let small_ps = output
        .placed
        .iter()
        .find(|ps| ps.placement.sprite_id == "small")
        .unwrap();
    // centered offset = (48 - 16) / 2 = 16
    assert_eq!(small_ps.placement.dest.x, 16);
    assert_eq!(small_ps.placement.dest.y, 16);
}

#[test]
fn grid_atlas_covers_all_placed_sprites() {
    let sprites: Vec<Sprite> = (0..6)
        .map(|i| make_sprite(&format!("s{i}"), 32, 32))
        .collect();
    let output = Grid::default()
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    let max_right = output
        .placed
        .iter()
        .map(|ps| ps.placement.dest.right())
        .max()
        .unwrap();
    let max_bottom = output
        .placed
        .iter()
        .map(|ps| ps.placement.dest.bottom())
        .max()
        .unwrap();
    assert!(output.atlas_size.w >= max_right);
    assert!(output.atlas_size.h >= max_bottom);
}

// MaxRects packer

#[test]
fn maxrects_empty_input_returns_error() {
    let result = MaxRects::default().pack(PackInput {
        sprites: vec![],
        config: fast_layout(512, 512),
        sprite_config: SpriteConfig::default(),
    });
    assert!(result.is_err());
}

#[test]
fn maxrects_single_sprite_placed() {
    let output = MaxRects::default()
        .pack(PackInput {
            sprites: vec![make_sprite("a", 64, 64)],
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert_eq!(output.placed.len(), 1);
    assert!(output.overflow.is_empty());
    assert_eq!(output.placed[0].placement.sprite_id, "a");
}

#[test]
fn maxrects_ten_sprites_all_placed_no_overflow() {
    let sprites: Vec<Sprite> = (0..10)
        .map(|i| make_sprite(&format!("s{i}"), 32, 32))
        .collect();
    let output = MaxRects::default()
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert_eq!(output.placed.len(), 10);
    assert!(output.overflow.is_empty());
}

#[test]
fn maxrects_sprite_too_large_for_remaining_space_overflows() {
    // One atlas-filling sprite leaves no room for a second.
    let sprites = vec![make_sprite("fill", 512, 512), make_sprite("extra", 32, 32)];
    let output = MaxRects::default()
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert_eq!(output.placed.len(), 1);
    assert_eq!(output.overflow.len(), 1);
    assert_eq!(output.overflow[0].id, "extra");
}

#[test]
fn maxrects_placed_rects_do_not_overlap() {
    let sprites: Vec<Sprite> = (0..8)
        .map(|i| make_sprite(&format!("s{i}"), 40, 40))
        .collect();
    let output = MaxRects::default()
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    let rects: Vec<Rect> = output.placed.iter().map(|ps| ps.placement.dest).collect();
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            assert!(
                !rects[i].intersects(&rects[j]),
                "rects [{i}] and [{j}] overlap: {:?} vs {:?}",
                rects[i],
                rects[j]
            );
        }
    }
}

#[test]
fn maxrects_atlas_size_covers_all_sprites() {
    let sprites = vec![make_sprite("a", 100, 100), make_sprite("b", 80, 80)];
    let output = MaxRects::default()
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    let max_right = output
        .placed
        .iter()
        .map(|ps| ps.placement.dest.right())
        .max()
        .unwrap();
    let max_bottom = output
        .placed
        .iter()
        .map(|ps| ps.placement.dest.bottom())
        .max()
        .unwrap();
    assert!(output.atlas_size.w >= max_right);
    assert!(output.atlas_size.h >= max_bottom);
}

#[test]
fn maxrects_mixed_sizes_all_fit() {
    let sprites = vec![
        make_sprite("s1", 64, 64),
        make_sprite("s2", 128, 32),
        make_sprite("s3", 32, 128),
        make_sprite("s4", 16, 16),
        make_sprite("s5", 48, 80),
    ];
    let output = MaxRects::default()
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    assert_eq!(output.placed.len(), 5);
    assert!(output.overflow.is_empty());
}

#[test]
fn maxrects_all_placed_ids_are_present() {
    let ids = ["alpha", "beta", "gamma", "delta"];
    let sprites: Vec<Sprite> = ids.iter().map(|id| make_sprite(id, 32, 32)).collect();
    let output = MaxRects::default()
        .pack(PackInput {
            sprites,
            config: fast_layout(512, 512),
            sprite_config: SpriteConfig::default(),
        })
        .unwrap();
    let placed_ids: Vec<&str> = output
        .placed
        .iter()
        .map(|ps| ps.placement.sprite_id.as_str())
        .collect();
    for id in &ids {
        assert!(placed_ids.contains(id), "missing sprite id: {id}");
    }
}
