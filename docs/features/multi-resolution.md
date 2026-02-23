# Multi-Resolution Variants

Scale variants let you generate atlas files at multiple resolutions in a single pass. A common setup produces `@1x` and `@2x` atlases from the same source sprites: the high-resolution atlas serves retina or high-DPI displays; the low-resolution one serves standard displays.

Each variant specifies a scale factor, an optional filename suffix, and a resampling mode. All variants are produced from the same loaded source sprites — images are only read from disk once.

## Usage

### CLI Flags

To produce a single scaled atlas:

```
fastpack pack sprites/ --scale 0.5 --suffix @1x
```

Multiple variants require a project file.

### `.fpsheet` Fields

```toml
[[variants]]
scale      = 2.0
suffix     = "@2x"
scale_mode = "smooth"

[[variants]]
scale      = 1.0
suffix     = "@1x"
scale_mode = "smooth"
```

`scale_mode = "smooth"` resamples with Lanczos3. Use it for photographic sprites or mixed content. `scale_mode = "fast"` uses nearest-neighbour and keeps hard pixel edges without blurring — use it for pixel art.

## Examples

With two variants and atlas name `"sprites"`, the output directory contains:

```
output/
  sprites@2x.png
  sprites@2x.json
  sprites@1x.png
  sprites@1x.json
```

Each data file's `meta.scale` reflects its variant's scale factor.

When multipack overflows on a variant, sheets are numbered before the suffix:

```
sprites@2x.png
sprites1@2x.png
sprites@2x.json
sprites1@2x.json
```

## Technical Notes

Each variant runs a full independent pipeline: sprites are scaled, trim rects are recomputed, the atlas is repacked, and the output is recompressed. Trim rects, nine-patch borders, and polygon vertices are all scaled proportionally so data file values are always in the variant's own pixel space.

Pivot points are normalized and scale-independent; they are copied to each variant unchanged.

A scale factor of exactly `1.0` skips resampling entirely and copies source pixels directly.

## TexturePacker Compatibility

TexturePacker calls these "Variants" and uses the same suffix convention. The `meta.scale` field in data files is a string (e.g. `"2"`, `"0.5"`) to match TexturePacker's serialization.
