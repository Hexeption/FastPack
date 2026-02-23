# JSON Array Export Format

The JSON Array format writes atlas metadata in the same structure as JSON Hash, but `frames` is a JSON array rather than an object. Each entry carries a `filename` field holding the sprite ID instead of using the sprite ID as an object key.

Use this format when your runtime iterates frames in order, or when a flat array is more convenient than a keyed object.

## Usage

### `.fpsheet` Fields

```toml
[output]
data_format = "json_array"
```

### CLI Flags

```
--data-format json_array
```

## Examples

Output for a two-sprite atlas:

```json
{
  "frames": [
    {
      "filename": "player/idle",
      "frame": { "x": 2, "y": 2, "w": 64, "h": 96 },
      "rotated": false,
      "trimmed": true,
      "spriteSourceSize": { "x": 4, "y": 0, "w": 64, "h": 96 },
      "sourceSize": { "w": 72, "h": 96 }
    },
    {
      "filename": "ui/button",
      "frame": { "x": 68, "y": 2, "w": 48, "h": 32 },
      "rotated": false,
      "trimmed": false,
      "spriteSourceSize": { "x": 0, "y": 0, "w": 48, "h": 32 },
      "sourceSize": { "w": 48, "h": 32 }
    }
  ],
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

When alias detection is on, duplicate sprites carry an `aliasOf` field pointing at the original sprite's ID. The `frame` rect is still valid — it points to the same atlas region.

## Technical Notes

Array order matches atlas packing order (largest-area-first). The order is stable across runs on the same input set.

All field semantics — `frame`, `rotated`, `trimmed`, `spriteSourceSize`, `sourceSize` — match the JSON Hash format. See [json-hash.md](json-hash.md) for field-level descriptions.

## TexturePacker Compatibility

Matches TexturePacker's JSON Array output. Engines using JSON Hash object-key lookup will not load this format — pick the right variant for your runtime loader.
