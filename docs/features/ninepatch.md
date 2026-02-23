# 9-Patch Metadata

9-patch metadata records the four border widths — top, right, bottom, left — of a stretchable UI sprite. The atlas packer does not alter how the sprite is packed; it only stores the border values in the data file. Your runtime reads those values to know which regions of the sprite can stretch and which must stay fixed.

## Usage

9-patch borders are set per sprite via the project file. There is no CLI flag.

### `.fpsheet` Fields

Add a `[[sprite_overrides]]` entry for each sprite that needs 9-patch metadata:

```toml
[[sprite_overrides]]
id         = "ui/button"
nine_patch = { top = 8, right = 8, bottom = 8, left = 8 }

[[sprite_overrides]]
id         = "ui/panel"
nine_patch = { top = 12, right = 20, bottom = 12, left = 20 }
```

Border values are in source pixels before any scale variant is applied. When a scale variant is used, FastPack scales the border values proportionally in that variant's data file.

## Examples

A `ui/button` sprite with `nine_patch = { top=8, right=8, bottom=8, left=8 }`:

**JSON Hash output:**

```json
"ui/button": {
  "frame": { "x": 2, "y": 2, "w": 64, "h": 32 },
  "rotated": false,
  "trimmed": false,
  "spriteSourceSize": { "x": 0, "y": 0, "w": 64, "h": 32 },
  "sourceSize": { "w": 64, "h": 32 },
  "ninePatch": { "top": 8, "right": 8, "bottom": 8, "left": 8 }
}
```

The `ninePatch` field is omitted for sprites that have no override.

## Technical Notes

`ninePatch` is written in camelCase in the JSON output, consistent with the other camelCase fields (`spriteSourceSize`, `sourceSize`).

Border values in the data file are in the variant's pixel space. A `@0.5x` scale variant writes borders that are half the project-file values.

## TexturePacker Compatibility

TexturePacker stores 9-patch borders in a `slices` array. FastPack uses a flat `ninePatch` object, which is more compact and widely used in game engines. If your engine expects the `slices` format, a small post-processing step is needed to remap the field.
