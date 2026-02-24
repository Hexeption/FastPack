# Premultiply Alpha

When premultiplied alpha is enabled, fastpack multiplies each RGB channel by the pixel's alpha value before writing the texture. A pixel with alpha 128 and red 200 becomes red 100 (200 × 128 / 255), alpha 128.

Most real-time renderers and GPU blending pipelines expect straight alpha (the default). Premultiplied alpha is useful when:

- The engine blends using `src_colour + (1 − src_alpha) × dst_colour` rather than the default `src_alpha × src_colour + (1 − src_alpha) × dst_colour`
- You use mipmapping and need correct colour bleed into transparent borders
- You export to DXT5/BC3 and want the colour endpoints to match what the engine actually renders

## CLI flag

```
--premultiply-alpha
```

The flag takes no value. Pass it to enable.

```
fastpack pack sprites/ --output out/ --premultiply-alpha
fastpack pack sprites/ --output out/ --premultiply-alpha --pixel-format rgba4444
```

## .fpsheet field

Premultiply alpha is not yet stored in `.fpsheet` project files. Set it via the CLI or the GUI toggle.

## Technical notes

The transform is applied after the atlas is composited and before dithering. Order matters: dithering operates on premultiplied values, which is what the GPU will read at runtime.

For fully opaque pixels (alpha 255) the multiplication is a no-op and fastpack skips the computation. Fully transparent pixels (alpha 0) are written as `[0, 0, 0, 0]`.

The transform is not reversible. Do not premultiply source PNGs before packing; let fastpack do it on the composited atlas.

## TexturePacker compatibility

TexturePacker's **Premultiply alpha** checkbox produces equivalent output for standard RGBA textures. When combined with a quantisation format such as RGBA4444, both tools dither after premultiplying.
