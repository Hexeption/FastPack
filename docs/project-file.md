# Project File (.fpsheet)

A `.fpsheet` file is a TOML document that stores all packer settings for a project. Instead of passing flags on every run, write them once to an `.fpsheet` and reference it with `--project`.

## Creating a Project File

Generate a default `.fpsheet` in the current directory:

```
fastpack init
```

Reference it when packing or watching:

```
fastpack pack --project atlas.fpsheet
fastpack watch --project atlas.fpsheet
```

CLI flags override project file values when both are provided.

## Full Example

```toml
# Input sources
[[sources]]
path   = "sprites"
filter = "**/*.png"

# Atlas layout
[layout]
max_width      = 4096
max_height     = 4096
border_padding = 2
shape_padding  = 2
allow_rotation = true
pack_mode      = "best"

# Sprite pre-processing
[sprites]
trim_mode      = "trim"
trim_threshold = 1
trim_margin    = 0
extrude        = 0
detect_aliases = true
default_pivot  = { x = 0.0, y = 0.0 }

# Output
[output]
name                = "atlas"
directory           = "output"
texture_format      = "png"
pixel_format        = "RGBA8888"
data_format         = "json_hash"
quality             = 95
premultiply_alpha   = false
texture_path_prefix = ""

# Algorithm
[algorithm]
type      = "max_rects"
heuristic = "best_short_side_fit"

# Scale variants
[[variants]]
scale      = 1.0
suffix     = ""
scale_mode = "smooth"

# Per-sprite overrides
[[sprite_overrides]]
id         = "ui/button"
pivot      = { x = 0.5, y = 0.5 }
nine_patch = { top = 8, right = 8, bottom = 8, left = 8 }
```

## Field Reference

### `[[sources]]`

Multiple `[[sources]]` blocks are allowed. Each adds a set of sprites to the pack.

**path** — root directory to search. Defaults to `"sprites"`.

**filter** — glob pattern relative to `path`. Defaults to `"**/*.png"`.

### `[layout]`

**max_width** — maximum atlas width in pixels. Default `4096`.

**max_height** — maximum atlas height in pixels. Default `4096`.

**fixed_width** — force an exact width, overriding `max_width`.

**fixed_height** — force an exact height, overriding `max_height`.

**border_padding** — pixels of empty space around the atlas edge. Default `2`.

**shape_padding** — pixels of empty space between sprites. Default `2`.

**allow_rotation** — allow 90° sprite rotation to improve fit. Default `true`.

**pack_mode** — `"fast"` uses the basic strip packer. `"good"` runs MaxRects in a single pass. `"best"` searches for the most compact width. Default `"best"`.

**force_square** — constrain atlas to square dimensions. Default `false`.

**size_constraint** — valid atlas dimension modes: `"any_size"`, `"pot"` (power of two), `"multiple_of_4"`, `"word_aligned"`. Default `"any_size"`.

### `[sprites]`

**trim_mode** — `"none"`, `"trim"`, `"crop"`, `"crop_keep_pos"`, or `"polygon"`. Default `"trim"`. See [trimming.md](features/trimming.md).

**trim_threshold** — alpha ≤ this value is treated as transparent. Range 0–255. Default `1`.

**trim_margin** — transparent pixels kept around the trim edge. Default `0`.

**extrude** — edge pixels to repeat outward before packing. Default `0`. See [extrude.md](features/extrude.md).

**detect_aliases** — deduplicate pixel-identical sprites. Default `true`. See [alias-detection.md](features/alias-detection.md).

**default_pivot** — fallback pivot for all sprites. Default `{ x = 0.0, y = 0.0 }`. See [pivot-points.md](features/pivot-points.md).

**common_divisor_x** — round trimmed width up to the nearest multiple. `0` disables. Default `0`.

**common_divisor_y** — round trimmed height up to the nearest multiple. `0` disables. Default `0`.

### `[output]`

**name** — base filename without extension. Default `"atlas"`.

**directory** — output directory path. Default `"output"`.

**texture_format** — `"png"`, `"jpg"`, or `"webp"`. Default `"png"`.

**pixel_format** — pixel encoding written to the data file. Default `"RGBA8888"`.

**data_format** — `"json_hash"`, `"json_array"`, `"phaser3"`, or `"pixijs"`. Default `"json_hash"`.

**quality** — lossy encoding quality, 0–100. Applies to JPEG and WebP. Default `95`.

**premultiply_alpha** — multiply RGB by alpha before encoding. Default `false`.

**texture_path_prefix** — string prepended to texture filenames in data files. Default `""`.

### `[algorithm]`

**MaxRects (default):**

```toml
[algorithm]
type      = "max_rects"
heuristic = "best_short_side_fit"
```

Available heuristics: `best_short_side_fit`, `best_long_side_fit`, `best_area_fit`, `bottom_left_rule`, `contact_point_rule`.

See [maxrects.md](algorithms/maxrects.md) for details.

**Grid:**

```toml
[algorithm]
type        = "grid"
cell_width  = 64
cell_height = 64
```

`cell_width = 0` and `cell_height = 0` auto-size to the largest sprite. See [grid.md](algorithms/grid.md).

**Basic:**

```toml
[algorithm]
type = "basic"
```

See [basic.md](algorithms/basic.md).

### `[[variants]]`

Each `[[variants]]` block produces an independent set of output files at the specified scale. See [multi-resolution.md](features/multi-resolution.md).

**scale** — scale factor. `0.5` halves all sprite dimensions; `2.0` doubles them. Default `1.0`.

**suffix** — appended to output filenames before the extension. Empty by default.

**scale_mode** — `"smooth"` uses Lanczos3; `"fast"` uses nearest-neighbour. Use `"fast"` for pixel art.

### `[[sprite_overrides]]`

Per-sprite metadata overrides. See [pivot-points.md](features/pivot-points.md) and [ninepatch.md](features/ninepatch.md).

**id** — sprite ID: relative path without extension, forward-slash separated.

**pivot** — `{ x = float, y = float }` normalized anchor point.

**nine_patch** — `{ top = int, right = int, bottom = int, left = int }` border widths in source pixels.

## TexturePacker Compatibility

The `.fpsheet` format is FastPack-specific TOML. TexturePacker uses its own `.tps` XML format. Settings map closely between the two tools but the file formats are not interchangeable.
