# PNG Optimization

The PNG compression backend encodes atlas textures as lossless PNG. The effort level is controlled by `pack_mode`: `fast` skips post-processing, `good` runs oxipng at moderate effort, `best` runs oxipng at maximum effort.

## Usage

### `.fpsheet` Fields

```toml
[output]
texture_format = "png"

[layout]
pack_mode = "good"   # fast | good | best
```

### CLI Flags

```
--texture-format png
--pack-mode <fast|good|best>
```

## Technical Notes

The pipeline runs in two passes. First, `image` encodes the composited atlas to PNG bytes using its built-in deflate encoder. Then, if `pack_mode` is `good` or `best`, those bytes are passed to oxipng for lossless re-compression.

`fast` skips oxipng entirely. Output size depends only on the `image` crate's default deflate settings.

`good` uses oxipng preset 3. Suitable for production builds where you want smaller files without paying the full optimization cost.

`best` uses oxipng preset 6 (maximum libdeflater compression). Produces the smallest possible lossless PNG. Slower, but worthwhile for shipped assets.

oxipng only reorders and refilters the pixel data — it never changes any pixel values. The output is bit-for-bit identical in decoded pixels to the `fast` output.

The `png` Cargo feature gates the oxipng dependency. When the feature is disabled, `fast` mode is used unconditionally regardless of the configured `pack_mode`.

## TexturePacker Compatibility

TexturePacker's "Optimize for size" option uses a similar two-pass approach with oxipng or similar tooling. FastPack's `best` preset produces comparable or smaller files for the same input.
