# MaxRects Packing Algorithm

MaxRects is the default packing algorithm. It produces dense atlases by tracking a list of maximal free rectangles and scoring candidate placements under a configurable heuristic.

## How It Works

The algorithm maintains a list of free rectangles covering all unoccupied atlas space. Initially the list contains one rectangle equal to the usable canvas (max dimensions minus border padding).

For each sprite (processed largest-area-first):

1. Score every free rectangle against the sprite dimensions using the active heuristic.
2. Place the sprite at the best-scoring position.
3. Split every free rectangle that overlaps the placed footprint (including shape padding) into up to four non-overlapping sub-rectangles.
4. Prune any free rectangle that is fully contained within another free rectangle.

Sprites that cannot fit in any remaining free rectangle are returned as overflow. In multipack mode, overflow sprites seed the next atlas.

## Heuristics

**best-short-side-fit** (default) — Minimises the shorter leftover dimension after placement. Tends to produce compact, roughly square packing.

**best-long-side-fit** — Minimises the longer leftover dimension. Leaves fewer tall or wide slivers.

**best-area-fit** — Minimises wasted area after placement. Good for mixed-size sprite sets.

**bottom-left-rule** — Places each sprite at the topmost then leftmost available position. Produces a predictable, scanline-style layout.

**contact-point-rule** — Maximises the perimeter of the placed sprite that borders already-placed sprites or atlas walls. Reduces internal fragmentation. *(Phase 2 — currently falls back to best-area-fit.)*

## `.fpsheet` Fields

```toml
[algorithm]
type       = "max_rects"
heuristic  = "best_short_side_fit"  # see heuristics above

[layout]
max_width       = 4096
max_height      = 4096
allow_rotation  = true
shape_padding   = 2
border_padding  = 2
size_constraint = "any_size"   # any_size | pot | multiple_of_4 | word_aligned
force_square    = false
```

## CLI Flags

```
--algorithm max-rects
--max-rects-heuristic <heuristic>    best-short-side-fit | best-long-side-fit | best-area-fit | bottom-left-rule | contact-point-rule
--allow-rotation                     (flag)
--shape-padding <n>
--border-padding <n>
--size-constraints <constraint>
--force-squared                      (flag)
```

## Technical Notes

**Shape padding** is handled by adding the padding value to the sprite's footprint width and height before scoring and splitting. The actual sprite rectangle (`dest`) is stored without padding; the gap appears as unused space between sprites.

**Border padding** offsets the initial free rectangle inward so all placed sprites respect the configured atlas edge margin.

**Atlas size** is computed after packing as the bounding box of all placed sprites plus border padding, then rounded up to satisfy `size_constraint`. If `force_square` is set, width and height are both raised to the larger of the two.

**Rotation** is tested when `allow_rotation` is true and the sprite is not square. The rotated candidate is scored independently. The orientation with the better score wins. A rotated sprite is blitted 90° clockwise.

**Free-rect pruning** is O(n²) per placement. For atlases with thousands of sprites this is the dominant cost. The benchmark suite covers this directly.

## TexturePacker Compatibility

MaxRects maps directly to TexturePacker's MaxRects algorithm. All five heuristics have the same names (except TexturePacker uses "Best" as a prefix). The `contact-point-rule` heuristic currently uses `best-area-fit` scoring internally; results will differ from TexturePacker's contact point implementation until Phase 2.
