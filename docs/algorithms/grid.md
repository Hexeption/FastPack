# Grid Packing Algorithm

The grid packer divides the atlas into uniform cells of equal size and places each sprite into the next available cell in row-major order (left to right, then top to bottom). Cell dimensions are either derived automatically from the largest sprite in the set or set explicitly.

Grid packing is the right choice for animation frame strips and tile sets where all sprites are the same size. Its regular layout simplifies runtime frame calculation — a loader can compute any frame's position from its index with a single multiply, without parsing a data file.

## Usage

### `.fpsheet` Fields

Let FastPack compute cell size from the sprite set (uses the largest sprite's dimensions):

```toml
[algorithm]
type        = "grid"
cell_width  = 0
cell_height = 0
```

Fix an explicit cell size:

```toml
[algorithm]
type        = "grid"
cell_width  = 64
cell_height = 64
```

### CLI Flags

There is no dedicated CLI flag for grid packing. Use a project file with `[algorithm] type = "grid"`.

## Examples

Twenty 32×32 sprites packed into a 256-pixel-wide atlas with `cell_width = 32, cell_height = 32`:

```
┌────────────────────────────────────────┐
│  0   1   2   3   4   5   6   7        │  ← row 0 (8 cells × 32px = 256px)
│  8   9  10  11  12  13  14  15        │  ← row 1
│ 16  17  18  19                        │  ← row 2
└────────────────────────────────────────┘
```

All cells are `32×32`. Sprites smaller than the cell are placed at the top-left of the cell; unused area is transparent.

## Technical Notes

When `cell_width` or `cell_height` is `0`, FastPack measures every sprite and uses the maximum width and height respectively. Sprites that are larger than the explicit or derived cell size will overflow their cell and overlap the next — this is a configuration error.

The number of columns is `floor(max_width / cell_width)`. The atlas height grows to fit all rows needed for the full sprite set.

Despite the regular layout, FastPack still writes a full data file with exact frame rects. Sprites smaller than the cell have correct `frame` and `sourceSize` values, and pivot/nine-patch overrides work the same as with other algorithms.

## TexturePacker Compatibility

TexturePacker calls this mode "Fixed Grid". Cell-size derivation and row-major placement order match FastPack's behavior.
