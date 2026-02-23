# Phaser 3 Export Format

The Phaser 3 format writes a single JSON file containing a `textures` array. Each entry in the array represents one atlas sheet and carries its own `image`, `size`, `scale`, and `frames` array. This matches the layout Phaser 3 expects from `scene.load.multiatlas()`.

Single-sheet packs still produce a `textures` array with one entry, so you can always load with `multiatlas` regardless of whether multipack is on.

## Usage

### `.fpsheet` Fields

```toml
[output]
data_format = "phaser3"
```

### CLI Flags

```
--data-format phaser3
```

## Examples

Single-sheet output:

```json
{
  "textures": [
    {
      "image": "atlas.png",
      "format": "RGBA8888",
      "size": { "w": 512, "h": 256 },
      "scale": 1.0,
      "frames": [
        {
          "filename": "player/idle",
          "frame": { "x": 2, "y": 2, "w": 64, "h": 96 },
          "rotated": false,
          "trimmed": true,
          "spriteSourceSize": { "x": 4, "y": 0, "w": 64, "h": 96 },
          "sourceSize": { "w": 72, "h": 96 }
        }
      ]
    }
  ],
  "meta": {
    "app": "FastPack",
    "version": "3.0"
  }
}
```

Multi-sheet (multipack) output writes two entries in `textures` but only one JSON file:

```json
{
  "textures": [
    {
      "image": "atlas.png",
      "format": "RGBA8888",
      "size": { "w": 4096, "h": 4096 },
      "scale": 1.0,
      "frames": [ "..." ]
    },
    {
      "image": "atlas1.png",
      "format": "RGBA8888",
      "size": { "w": 1024, "h": 512 },
      "scale": 1.0,
      "frames": [ "..." ]
    }
  ],
  "meta": { "app": "FastPack", "version": "3.0" }
}
```

Loading in Phaser 3:

```js
this.load.multiatlas('sprites', 'atlas.json', 'assets/');
```

## Technical Notes

When multipack is active, the Phaser 3 exporter combines all sheets for a scale variant into a single JSON file. Secondary sheets do not get their own data files; only one `.json` is written per variant.

`scale` in each `textures` entry reflects the `ScaleVariant.scale` used during packing. Phaser uses this to apply sub-pixel corrections when mixing `@1x` and `@2x` atlases.

Frame fields use `camelCase` (`spriteSourceSize`, `sourceSize`) to match Phaser's parser.

## TexturePacker Compatibility

TexturePacker also produces this format when targeting Phaser 3. The `meta.app` field reads `"FastPack"` instead of `"TexturePacker"`; Phaser ignores the `app` field.
