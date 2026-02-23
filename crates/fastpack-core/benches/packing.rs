use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use fastpack_core::{
    algorithms::{
        basic::Basic,
        grid::Grid,
        maxrects::MaxRects,
        packer::{PackInput, Packer},
    },
    types::{
        config::{LayoutConfig, PackMode, SpriteConfig},
        rect::Size,
        sprite::Sprite,
    },
};
use image::{DynamicImage, Rgba, RgbaImage};
use std::path::PathBuf;

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

fn make_sprites(n: usize) -> Vec<Sprite> {
    (0..n)
        .map(|i| make_sprite(&format!("s{i}"), 32, 32))
        .collect()
}

fn layout() -> LayoutConfig {
    LayoutConfig {
        max_width: 4096,
        max_height: 4096,
        border_padding: 0,
        shape_padding: 0,
        allow_rotation: false,
        force_square: false,
        pack_mode: PackMode::Fast,
        ..LayoutConfig::default()
    }
}

fn bench_maxrects(c: &mut Criterion) {
    let mut group = c.benchmark_group("maxrects");
    for n in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                MaxRects::default()
                    .pack(PackInput {
                        sprites: black_box(make_sprites(n)),
                        config: layout(),
                        sprite_config: SpriteConfig::default(),
                    })
                    .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic");
    for n in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                Basic
                    .pack(PackInput {
                        sprites: black_box(make_sprites(n)),
                        config: layout(),
                        sprite_config: SpriteConfig::default(),
                    })
                    .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid");
    for n in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                Grid::default()
                    .pack(PackInput {
                        sprites: black_box(make_sprites(n)),
                        config: layout(),
                        sprite_config: SpriteConfig::default(),
                    })
                    .unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_maxrects, bench_basic, bench_grid);
criterion_main!(benches);
