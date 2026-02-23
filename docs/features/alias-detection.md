# Alias Detection

When two source sprites are pixel-for-pixel identical, alias detection deduplicates them: only one copy is packed into the atlas. All duplicates point back to the same atlas frame via an `aliasOf` field in the data file. This shrinks atlas area without changing the public sprite ID set — every sprite ID still resolves correctly at runtime.

Alias detection is on by default.

## Usage

### CLI Flags

There is no dedicated CLI flag. Alias detection is always on unless disabled in a project file.

### `.fpsheet` Fields

```toml
[sprites]
detect_aliases = true   # default
```

Set `detect_aliases = false` to pack every sprite independently even if some are identical.

## Examples

Two sprites `icons/star_gold.png` and `icons/star_gold_copy.png` have identical pixels. With alias detection on:

**JSON Hash output:**

```json
{
  "frames": {
    "icons/star_gold": {
      "frame": { "x": 2, "y": 2, "w": 32, "h": 32 },
      "rotated": false,
      "trimmed": false,
      "spriteSourceSize": { "x": 0, "y": 0, "w": 32, "h": 32 },
      "sourceSize": { "w": 32, "h": 32 }
    },
    "icons/star_gold_copy": {
      "frame": { "x": 2, "y": 2, "w": 32, "h": 32 },
      "rotated": false,
      "trimmed": false,
      "spriteSourceSize": { "x": 0, "y": 0, "w": 32, "h": 32 },
      "sourceSize": { "w": 32, "h": 32 },
      "aliasOf": "icons/star_gold"
    }
  }
}
```

Both IDs are present. `icons/star_gold_copy` shares the same `frame` rect and carries an `aliasOf` field. The atlas texture only contains the pixels once.

## Technical Notes

Equality is determined by a 64-bit xxHash of the raw RGBA pixel data, computed after loading and before trimming. Two sprites match if and only if their hashes are equal.

The canonical frame (the one that occupies the atlas rect) is chosen deterministically: the sprite that sorts first alphabetically by ID becomes the primary; all others become aliases of it.

Metadata fields (pivot, nine-patch) on alias entries are written independently — an alias can carry its own pivot even though it shares pixels with another sprite.

## TexturePacker Compatibility

TexturePacker also outputs an `aliasOf` field. FastPack uses the camelCase `aliasOf` key that matches TexturePacker's JSON Hash exporter.
