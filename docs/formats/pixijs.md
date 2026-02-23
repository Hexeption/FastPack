# PixiJS Export Format

The PixiJS format writes atlas metadata in JSON Hash layout, which is what PixiJS expects. There is no structural difference from the standard `json_hash` output — this format is provided as a named alias so project files can be explicit about which engine they target.

## Usage

### `.fpsheet` Fields

```toml
[output]
data_format = "pixijs"
```

### CLI Flags

```
--data-format pixijs
```

## Examples

Output is identical to JSON Hash. See [json-hash.md](json-hash.md) for a full example.

Loading in PixiJS v7+:

```js
await PIXI.Assets.load('atlas.json');
const sprite = PIXI.Sprite.from('player/idle');
```

Loading in PixiJS v6 and earlier:

```js
PIXI.Loader.shared.add('atlas', 'atlas.json').load(() => {
  const sprite = new PIXI.Sprite(PIXI.Texture.from('player/idle'));
});
```

## Technical Notes

The PixiJS exporter delegates directly to `JsonHashExporter`. Choosing `pixijs` vs `json_hash` produces byte-identical output; the distinction is purely for clarity in project files.

PixiJS expects sprite IDs as the frame keys, which is exactly what FastPack writes. Sprite IDs use forward slashes as path separators on all platforms.

## TexturePacker Compatibility

TexturePacker's PixiJS preset also outputs JSON Hash. Output from FastPack loads without modification in PixiJS projects previously built with TexturePacker.
