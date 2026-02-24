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

Available scale modes:

`smooth` resamples with Lanczos3. Use it for photographic sprites or mixed content.

`fast` uses nearest-neighbour. Hard pixel edges, no blurring. Good for pixel art at non-integer factors.

`scale2x` applies the EPX / Scale2x algorithm. Each source pixel expands to a 2×2 block, with corners filled by adjacent cardinal neighbours when they form a clean edge. Intended for pixel art scaled to exactly 2×, but works at any factor: the 2× intermediate is then resampled with nearest-neighbour to reach the target size.

`scale3x` is the 3× EPX variant. Same idea as `scale2x`, producing a 3×3 block per source pixel. Use it for 3× pixel art upscaling.

`hq2x` uses an edge-aware blend. It is a simplified version of the HQ2x algorithm: corner pixels blend towards matching cardinal neighbours to smooth diagonal edges in pixel art without blurring axis-aligned edges. Produces softer output than `scale2x` at 2×.

`eagle` applies the Eagle 2× algorithm. Each corner of the 2×2 output block picks the diagonal neighbour's colour when that neighbour also matches both adjacent cardinal neighbours. Sharper than `hq2x`, similar to `scale2x` but with different edge detection logic.

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

Pixel art modes (`scale2x`, `scale3x`, `hq2x`, `eagle`) apply their integer upscaler first, then resize to the exact target dimensions with nearest-neighbour if the factor does not align with the algorithm's native multiplier. For example, `scale2x` at factor `4.0` produces a 2× EPX intermediate and then doubles it again with nearest-neighbour to reach 4×. Metadata (trim rects, nine-patch borders, polygon vertices) is always scaled by the requested factor, not by the algorithm's native multiplier.

## TexturePacker Compatibility

TexturePacker calls these "Variants" and uses the same suffix convention. The `meta.scale` field in data files is a string (e.g. `"2"`, `"0.5"`) to match TexturePacker's serialization.
