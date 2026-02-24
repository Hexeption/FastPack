# FastPack

[![crates.io](https://img.shields.io/crates/v/fastpack.svg)](https://crates.io/crates/fastpack)
[![CI](https://github.com/Hexeption/FastPack/actions/workflows/ci.yml/badge.svg)](https://github.com/Hexeption/FastPack/actions/workflows/ci.yml)
[![rustc 1.85+](https://img.shields.io/badge/rustc-1.85%2B-orange.svg)](https://www.rust-lang.org)

Texture atlas packer written in Rust. Native GUI as the primary interface, plus a full CLI and TUI. Designed as an open-source replacement for TexturePacker.

## Features

**Packing**
- MaxRects (5 heuristics), Grid, and Basic strip algorithms
- Trim modes: None, Trim, Crop, CropKeepPos, Polygon (convex hull)
- Extrusion, rotation, nine-patch metadata, pivot points
- Alias detection — deduplicates pixel-identical sprites
- Multipack — overflow sprites across multiple sheets
- Multi-resolution scale variants with per-variant suffix

**Export**
- JSON Hash, JSON Array, Phaser 3, PixiJS
- PNG (oxipng lossless), JPEG (mozjpeg), WebP, lossy PNG (imagequant)

**GUI**
- Real-time atlas preview
- Collapsible sprite tree with thumbnail previews
- Watch mode — repacks on file change
- `.fpsheet` project files (TOML)
- Multi-language UI

## Install

Download the installer for your platform from the [releases page](https://github.com/Hexeption/FastPack/releases):

- **Windows** — `fastpack-windows-x86_64.msi`
- **macOS (Apple Silicon)** — `fastpack-macos-aarch64.dmg`
- **macOS (Intel)** — `fastpack-macos-x86_64.dmg`
- **Linux** — `fastpack-linux-x86_64.tar.gz`

**macOS note:** The app is not code-signed, so macOS will show "FastPack is damaged and can't be opened." after mounting the DMG. Run this once after copying the app to `/Applications`:

```sh
xattr -cr /Applications/FastPack.app
```

Or install from crates.io:

```sh
cargo install fastpack
```

## Usage

```sh
# Open the GUI (default when no subcommand is given)
fastpack

# Pack a directory of sprites
fastpack pack sprites/ --output output/

# Pack with options
fastpack pack sprites/ --output output/ \
  --max-width 2048 --max-height 2048 \
  --trim-mode trim \
  --data-format phaser3 \
  --allow-rotation

# Load settings from a project file
fastpack pack --project atlas.fpsheet

# Watch for changes and repack automatically
fastpack watch sprites/ --output output/

# Split an atlas back into individual sprites
fastpack split atlas.png atlas.json --output-dir sprites/

# Generate a default project file
fastpack init --output atlas.fpsheet
```

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
detect_aliases = true

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

## Building from Source

Requires Rust 1.85+.

```sh
git clone https://github.com/Hexeption/FastPack
cd FastPack
cargo build --release -p fastpack
```

The binary is at `target/release/fastpack`.

## License

Licensed under [MIT](LICENSE).
