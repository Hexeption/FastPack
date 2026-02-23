# Pivot Points

A pivot point is a normalized (0.0–1.0) coordinate within a sprite that acts as its anchor at runtime. Engines use the pivot when positioning, rotating, or scaling a sprite: `(0, 0)` is the top-left corner, `(0.5, 0.5)` is the centre, `(1, 1)` is the bottom-right.

FastPack stores pivot metadata in the atlas data file per sprite. The packer itself does not use the pivot — it is purely informational for your runtime.

## Usage

### CLI Flags

Set a default pivot for every sprite in the pack:

```
fastpack pack sprites/ --pivot-x 0.5 --pivot-y 0.5
```

### `.fpsheet` Fields

Set a default pivot in the project file:

```toml
[sprites]
default_pivot = { x = 0.5, y = 0.5 }
```

Override the pivot for individual sprites:

```toml
[[sprite_overrides]]
id    = "player/character"
pivot = { x = 0.5, y = 1.0 }   # feet anchor

[[sprite_overrides]]
id    = "fx/explosion"
pivot = { x = 0.5, y = 0.5 }
```

Per-sprite overrides take priority over the default.

## Examples

**JSON Hash output:**

```json
"player/character": {
  "frame": { "x": 2, "y": 2, "w": 48, "h": 96 },
  "rotated": false,
  "trimmed": true,
  "spriteSourceSize": { "x": 8, "y": 0, "w": 48, "h": 96 },
  "sourceSize": { "w": 64, "h": 96 },
  "pivot": { "x": 0.5, "y": 1.0 }
},
"fx/explosion": {
  "frame": { "x": 52, "y": 2, "w": 64, "h": 64 },
  "rotated": false,
  "trimmed": false,
  "spriteSourceSize": { "x": 0, "y": 0, "w": 64, "h": 64 },
  "sourceSize": { "w": 64, "h": 64 },
  "pivot": { "x": 0.5, "y": 0.5 }
}
```

The `pivot` field is omitted when the default pivot is `{ x: 0, y: 0 }` and no override is set.

## Technical Notes

Pivot values are normalized relative to `sourceSize` (the original image dimensions before trimming), not the packed frame. A pivot of `(0.5, 1.0)` on a 64×96 sprite always refers to the horizontal centre of the original at its bottom edge, regardless of trim.

Pivot values are scale-independent normalized coordinates. They are copied to each scale variant unchanged.

## TexturePacker Compatibility

TexturePacker writes pivot coordinates under the `pivot` key with the same `{ x, y }` float structure. Output from FastPack is compatible with engines that already read TexturePacker pivot data.
