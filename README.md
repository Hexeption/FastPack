# FastPack

[![crates.io](https://img.shields.io/crates/v/fastpack.svg)](https://crates.io/crates/fastpack)
[![CI](https://github.com/Hexeption/FastPack/actions/workflows/ci.yml/badge.svg)](https://github.com/Hexeption/FastPack/actions/workflows/ci.yml)
[![rustc 1.90+](https://img.shields.io/badge/rustc-1.90%2B-orange.svg)](https://www.rust-lang.org)

Texture atlas packer written in Rust. Ships as a Tauri desktop app and a headless CLI. Designed as an open-source replacement for TexturePacker.

<p align="center">
  <img src="https://github.com/user-attachments/assets/fe9df165-6800-465f-801d-b1655536edbf" alt="FastPack desktop app showing the sprite list, packed atlas preview, and output settings" />
</p>

## Features

**Packing**
- MaxRects (5 heuristics), Grid, and Basic strip algorithms
- Trim modes: None, Trim, Crop, CropKeepPos, Polygon (convex hull)
- Extrusion, rotation, nine-patch metadata, pivot points
- Reduce border artifacts (alpha bleeding) — fills transparent pixels with neighbouring colours to avoid dark fringes under bilinear filtering
- Alias detection — deduplicates pixel-identical sprites
- Multipack — overflow sprites across multiple sheets
- Multi-resolution scale variants with per-variant suffix
- TexturePacker-style sprite naming — keep file extensions, prepend folder names

**Export**
- Data formats: JSON Hash, JSON Array, Phaser 3, PixiJS
- Texture formats: PNG (oxipng lossless), JPEG (mozjpeg), WebP, DXT1 (BC1), DXT5 (BC3)
- Pixel formats: RGBA8888, RGB888, RGB565, RGBA4444, RGBA5551, Alpha8

**Desktop app**
- Real-time atlas preview
- Collapsible sprite tree with thumbnail previews
- Watch mode — repacks on file change
- `.fpsheet` project files (TOML)
- Nine-patch and pivot editors per sprite
- Drag-and-drop folders and project files
- Multi-language UI

## Install

Download the desktop app for your platform from the [releases page](https://github.com/Hexeption/FastPack/releases):

- **Windows** — `fastpack-windows-x86_64-setup.exe`
- **macOS (Apple Silicon)** — `fastpack-macos-aarch64.dmg`
- **macOS (Intel)** — `fastpack-macos-x86_64.dmg`
- **Linux** — `FastPack-x86_64.AppImage`

Or install the CLI from crates.io:

```sh
cargo install fastpack
```

## CLI Usage

```sh
# Pack a directory of sprites
fastpack pack sprites/ --output output/

# Pack with options
fastpack pack sprites/ --output output/ \
  --max-width 2048 --max-height 2048 \
  --trim-mode trim \
  --data-format phaser3 \
  --allow-rotation \
  --multipack

# TexturePacker-style frame names: sprites/hero/run_01.png → "hero/run_01.png"
fastpack pack sprites/hero --output output/ --keep-extension --prepend-folder-name

# Load settings from a project file
fastpack pack --project atlas.fpsheet

# Watch for changes and repack automatically
fastpack watch sprites/ --output output/

# Split an atlas back into individual sprites
fastpack split atlas.png atlas.json --output-dir sprites/

# Generate a default project file
fastpack init --output atlas.fpsheet
```

Run `fastpack <subcommand> --help` for the full flag list.

Frame names default to the path relative to the source folder without the
file extension (`run/run_01`). `--keep-extension` keeps the extension and
`--prepend-folder-name` prepends the source folder's own name; both only change
the names written to data files. Per-sprite overrides and `excludes` in a
project file always use the extension-less relative path.

## Project File

Settings live in a `.fpsheet` TOML file:

```toml
[meta]
version = "1"

[output]
name = "atlas"
directory = "output/"
texture_format = "png"
pixel_format = "rgba8888"
data_format = "json_hash"
quality = 95

[layout]
max_width = 4096
max_height = 4096
size_constraint = "pot"
force_square = false
allow_rotation = true
pack_mode = "good"
border_padding = 2
shape_padding = 2

[sprites]
trim_mode = "trim"
trim_threshold = 1
extrude = 0
alpha_bleed = false
detect_aliases = true
keep_extension = false      # true: frame names keep ".png" (TexturePacker "Trim sprite names" off)
prepend_folder_name = false # true: source folder name is prepended ("hero/run_01")

[algorithm]
type = "max_rects"
heuristic = "best_short_side_fit"

[[variants]]
scale = 1.0
suffix = "@1x"
mode = "smooth"

[[sources]]
path = "sprites/"
filter = "**/*.png"
```

## Export Formats

`data_format` in the project file or `--data-format` on the CLI accepts:

- `json_hash` — TexturePacker-compatible JSON with frames as an object keyed by sprite ID. Default.
- `json_array` — Same structure but frames as an array, each entry with a `filename` field.
- `phaser3` — Single JSON file with a `textures` array. Compatible with `scene.load.multiatlas()`.
- `pixijs` — JSON Hash format compatible with PixiJS sprite sheet loaders.
- `cocos2d` — Cocos2d-x property list (plist format 3). Frame names carry a `.png` suffix, as TexturePacker writes them for `SpriteFrameCache`.
- `sparrow` — Starling / Sparrow `TextureAtlas` XML. Frame names carry a `.png` suffix; rotated sprites are stored 90° clockwise.
- `libgdx` — libGDX `TextureAtlas` (`.atlas`) using `bounds`/`offsets`/`rotate`, with `split` for nine-patch sprites. Region names have no extension; rotated sprites are stored 90° counter-clockwise, as libGDX expects. Multipack sheets share one file.
- `spine` — Spine 4 `.atlas`. Same layout as `libgdx` in Spine's compact style (`rotate:90`); region names have no extension. Multipack sheets share one file.
- `godot` — Godot 4 `.tpsheet` JSON for CodeAndWeb's TexturePacker importer plugin (`textures[]` with `region`/`margin` per sprite). Multipack sheets share one file. Sprite rotation is disabled because Godot's `AtlasTexture` cannot show rotated regions.
- `css` — CSS stylesheet with one class per sprite (`<span class="sprite hero"></span>`). Sprite ids become valid, unique class names (`ui/button` → `ui-button`). Multipack sheets share one file. Sprite rotation is disabled; trimmed sprites display at their trimmed size.

## Building from Source

Requires Rust 1.90+.

**CLI only:**

```sh
git clone https://github.com/Hexeption/FastPack
cd FastPack
cargo build --release -p fastpack
```

The binary is at `target/release/fastpack`.

**Desktop app:**

The Tauri app also requires Node.js and pnpm. From the `crates/fastpack-tauri` directory:

```sh
pnpm install
pnpm tauri build
```

The installer is placed under `src-tauri/target/release/bundle/`.

**Contributing:**

Enable the pre-commit hook, which runs the same checks as CI (`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`) and blocks the commit if any fail:

```sh
git config core.hooksPath .githooks
```

CI uses the latest stable Rust, so the hook also asks you to `rustup update stable` when you are behind.

## License

Licensed under [MIT](LICENSE).
