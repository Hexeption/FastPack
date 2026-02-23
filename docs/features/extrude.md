# Sprite Extrusion

Extrusion repeats the outermost row or column of pixels of each sprite outward by a configurable number of pixels before packing. The repeated pixels fill the spacing around each sprite in the atlas.

This prevents color bleeding — the visual artefact where, at scaled or sub-pixel render sizes, the GPU samples past a sprite's boundary and picks up the neighbouring transparent or mis-colored pixels.

## Usage

Extrusion is set via the project file. There is no dedicated CLI flag.

### `.fpsheet` Fields

```toml
[sprites]
extrude = 1   # repeat edge pixels 1px outward (default: 0)
```

Higher values are rarely needed. `1` is enough for bilinear filtering; `2` is sometimes used with mipmapped sprites.

## Examples

A 4×4 checkerboard sprite extruded by 1 pixel grows to 6×6 in the atlas. The 1-pixel border around it is a copy of the sprite's edge pixels.

```
Original (4×4)     With extrude=1 in atlas (6×6)
B W B W            B B W B W W
W B W B            B B W B W W
B W B W            W W B W B B
W B W B            B B W B W W
                   W W B W B B
                   W W B W B B
```

The `frame` rect in the data file still points at the inner 4×4 region. The runtime does not need to know extrusion happened.

## Technical Notes

Extrusion is applied during pre-processing, after trimming and before packing. The sprite's pixel buffer grows by `extrude` pixels on every side; the packer sees the enlarged size.

`frame`, `spriteSourceSize`, and `sourceSize` in the data file all reflect the original (pre-extrusion) dimensions. The atlas texture contains the extruded version, but the frame rect selects the inner region that excludes the duplicated border.

At the default shape padding of 2, an `extrude` value of 2 means the duplicated edge sits exactly adjacent to the padding gap — no atlas space is wasted between the extruded border and the next sprite.

Extrusion does not change trim behavior: if trimming is active, the sprite is trimmed first, then the trimmed result is extruded.

## TexturePacker Compatibility

TexturePacker calls this setting "Extrude" and applies the same edge-repeat logic. Atlas output with matching `extrude` values is visually and functionally equivalent.
