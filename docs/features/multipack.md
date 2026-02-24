# Multi-Pack

Multipack splits sprites across multiple atlas sheets when they do not all fit within the configured maximum dimensions. Overflow sprites go into additional sheets named with an incrementing index suffix: `atlas.png`, `atlas1.png`, `atlas2.png`, and so on.

Each sheet gets its own data file with the same naming pattern, except when using the Phaser 3 format, which combines all sheets into one JSON file.

## Usage

### CLI Flags

```
fastpack pack sprites/ --multipack
fastpack pack sprites/ --multipack --max-width 2048 --max-height 2048
```

### `.fpsheet` Fields

```toml
[layout]
max_width  = 2048
max_height = 2048
```

Enable via CLI `--multipack` or set in the project file. When no `--multipack` flag is given on the CLI, packing stops with an error if sprites overflow a single sheet.

### GUI

Open the **Texture** section of the settings panel and check **Multipack**. The project re-packs immediately.

When the result has more than one sheet, all sheets appear side by side in the atlas preview. Pan and zoom apply to the entire group. Each sheet shows a `Sheet N: W×H` label. The sprite list includes frames from all sheets. Clicking a sprite on any sheet selects it.

## Examples

With 600 sprites that do not fit on a 2048×2048 atlas:

```
output/
  atlas.png     # first sheet
  atlas.json
  atlas1.png    # overflow sheet
  atlas1.json
```

Loading both sheets:

```js
// PixiJS
await PIXI.Assets.load(['atlas.json', 'atlas1.json']);

// Phaser 3 (single combined JSON when using --data-format phaser3)
this.load.multiatlas('sprites', 'atlas.json');
```

## Technical Notes

Sprites are sorted by area descending and packed into the first sheet. When a sprite does not fit in the remaining space, it becomes the first sprite of the next sheet. Packing continues on the new sheet from the overflow list.

Sheet index suffixes start at `1` for the second sheet. The first sheet has no suffix.

When `--data-format phaser3` is used, only one JSON file is written (named after the first sheet). It contains a `textures` array with one entry per sheet, so `scene.load.multiatlas()` loads all sheets from a single call.

Scale variants each run an independent multipack sequence, producing `atlas@2x.png`, `atlas1@2x.png`, etc. when a suffix is configured.

## TexturePacker Compatibility

TexturePacker calls this "Multipack" and uses the same sheet numbering convention. Data files from FastPack load in the same way.
