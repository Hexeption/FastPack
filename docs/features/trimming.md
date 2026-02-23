# Sprite Trimming

Trimming removes transparent borders from sprites before packing. Smaller packed rects mean a denser atlas and less wasted GPU texture memory. The original sprite dimensions and the trim offset are stored in the data file so engines can reconstruct the sprite at its correct screen position.

## Trim Modes

**none** — No pixels are removed. The full source image is packed as-is. No trim metadata is written to the data file.

**trim** — The transparent border is stripped. The packed rect is tight around the opaque region. `spriteSourceSize` and `sourceSize` in the data file give the original dimensions and offset needed to restore position.

**crop** — Same pixel crop as `trim`. Intended for use cases where offset restoration is handled outside the data file.

**crop-keep-pos** — Same crop, but the trim offset in the data file may be negative. Used when the sprite's registration point must be held fixed relative to the original canvas even after cropping.

**polygon** — Crops to the bounding box of the convex hull of opaque pixels. The hull vertices are stored in the data file for engines that support tight mesh trimming. *(Hull computation is a Phase 3 feature; the current build crops to the bounding box only.)*

## `.fpsheet` Fields

```toml
[sprites]
trim_mode       = "trim"   # none | trim | crop | crop-keep-pos | polygon
trim_threshold  = 1        # pixels with alpha <= this value are treated as transparent (0–255)
trim_margin     = 0        # transparent pixels to keep around the trimmed edge
common_divisor_x = 0       # round trimmed width up to the nearest multiple (0 = disabled)
common_divisor_y = 0       # round trimmed height up to the nearest multiple (0 = disabled)
```

## CLI Flags

```
--trim-mode <mode>          none | trim | crop | crop-keep-pos | polygon
--trim-threshold <n>        0–255, default 1
--trim-margin <n>           pixels, default 0
```

## Examples

Keep a 2-pixel transparent margin around every trimmed sprite:

```toml
[sprites]
trim_mode    = "trim"
trim_margin  = 2
```

Ensure all trimmed widths are multiples of 4 (required for some texture formats):

```toml
[sprites]
trim_mode        = "trim"
common_divisor_x = 4
common_divisor_y = 4
```

## Technical Notes

The bounding box scan visits every pixel once. For each pixel, the alpha channel is compared against `trim_threshold`. Pixels with alpha strictly greater than the threshold are considered opaque.

After the bounding box is found, `trim_margin` is added on all four sides and the result is clamped to the image dimensions. `common_divisor_x/y` then expands the width/height to the next valid multiple, expanding toward the right/bottom edge and clamping again.

The `Sprite::original_size` field records the pre-trim dimensions and is never changed by this stage.

## TexturePacker Compatibility

FastPack's `trim` mode maps to TexturePacker's "Trim" sprite mesh type. TexturePacker's "Crop" maps to `crop`. The `trim_threshold` is equivalent to TexturePacker's alpha threshold setting. `trim_margin` is equivalent to TexturePacker's "border padding" on the sprite level (not to be confused with atlas-level border padding).
