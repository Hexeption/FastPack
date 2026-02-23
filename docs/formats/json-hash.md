# JSON Hash Export Format

The JSON Hash format writes atlas metadata as a single JSON object. Each sprite frame is a key in the top-level `frames` object. The key is the sprite ID — the source path relative to the sprite root, without file extension.

This is the default output format and matches TexturePacker's JSON Hash layout exactly.

## Usage

### `.fpsheet` Fields

```toml
[output]
data_format = "json_hash"
```

### CLI Flags

```
--data-format json_hash
```

## Examples

Output for a two-sprite atlas:

```json
{
  "frames": {
    "player/idle": {
      "frame": { "x": 2, "y": 2, "w": 64, "h": 96 },
      "rotated": false,
      "trimmed": true,
      "spriteSourceSize": { "x": 4, "y": 0, "w": 64, "h": 96 },
      "sourceSize": { "w": 72, "h": 96 }
    },
    "ui/button": {
      "frame": { "x": 68, "y": 2, "w": 48, "h": 32 },
      "rotated": false,
      "trimmed": false,
      "spriteSourceSize": { "x": 0, "y": 0, "w": 48, "h": 32 },
      "sourceSize": { "w": 48, "h": 32 }
    }
  },
  "meta": {
    "app": "FastPack",
    "version": "1.0",
    "image": "atlas.png",
    "format": "RGBA8888",
    "size": { "w": 256, "h": 128 },
    "scale": "1"
  }
}
```

## Technical Notes

**frame** holds the rectangle in the atlas texture: `x`, `y`, `w`, `h` in pixels.

**spriteSourceSize** is the frame's position and size relative to the original source image. `x` and `y` are the offset of the trimmed region within the original. For `CropKeepPos` trim mode these can be negative. `w` and `h` match `frame.w` and `frame.h` (the packed size, not the source size).

**sourceSize** is the full original image dimensions before any trimming.

**rotated** is `true` when the sprite was rotated 90° clockwise to fit. When reading a rotated frame, swap `w` and `h` from `frame` to get the display dimensions.

**trimmed** is `true` when the sprite was packed with `TrimMode::Trim`, `Crop`, `CropKeepPos`, or `Polygon`. It is `false` for `TrimMode::None`.

**meta.scale** reflects the `ScaleVariant.scale` used during packing, serialized as a string (e.g. `"1"`, `"0.5"`, `"2"`).

Frame keys are inserted in atlas frame order (largest-area-first processing order, as produced by the packer). JSON object key ordering is not guaranteed by spec; load-time parsers should treat `frames` as an unordered map.

## TexturePacker Compatibility

The output structure matches TexturePacker's JSON Hash format. Sprite IDs use forward slashes as path separators regardless of OS. The `meta.app` field reads `"FastPack"` instead of `"TexturePacker"`, which engines ignoring the `app` field will not notice.

Engines that parse `spriteSourceSize.x`/`.y` as signed integers handle `CropKeepPos` negative offsets. Engines expecting only unsigned values may misread them; check your engine's loader before using `CropKeepPos`.
