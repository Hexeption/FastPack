# Basic Packing Algorithm

The basic packer places sprites in horizontal strips. Starting from the top-left, it advances a cursor left-to-right across the row. When a sprite does not fit in the remaining row width, the cursor drops to the start of a new row below. Sprites within a row are top-aligned.

Basic packing is fast but makes no attempt to minimize atlas area. Use it when pack speed is the priority, or as a quick sanity-check mode during development.

## Usage

### CLI Flags

```
fastpack pack sprites/ --pack-mode fast
```

`--pack-mode fast` selects the basic packer. `good` and `best` both use MaxRects.

### `.fpsheet` Fields

```toml
[algorithm]
type = "basic"
```

Or via the layout shorthand:

```toml
[layout]
pack_mode = "fast"
```

## Examples

Five sprites of varying sizes packed with the basic algorithm:

```
┌──────────────────────────────┐
│ [64×96] [48×48] [32×32]  gap │  ← row 1
│                              │
│ [128×64] [16×16]         gap │  ← row 2
└──────────────────────────────┘
```

Row height equals the tallest sprite in that row. Shorter sprites leave vertical whitespace below them.

## Technical Notes

Sprites are sorted by height descending before placement. This reduces wasted vertical space within rows by putting tall sprites together.

The atlas width is the full `max_width` (default 4096); the basic packer does not search for a compact atlas width the way MaxRects does. The atlas height grows one row at a time until all sprites are placed.

Basic packing does not support multipack overflow. If sprites cannot fit within `max_width × max_height`, the pack fails with an error.

## TexturePacker Compatibility

TexturePacker's "Basic" algorithm uses the same row-strip approach. Atlas dimensions may differ because TexturePacker searches for an optimal width; FastPack's basic packer always uses the full `max_width`.
