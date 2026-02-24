# Pixel Format

FastPack packs to RGBA8888 by default. Use `--pixel-format` to quantize the atlas to a lower bit depth before writing to disk.

## Overview

Lower bit depths reduce file size and VRAM usage. FastPack applies Floyd-Steinberg error diffusion dithering when quantizing, which spreads quantization error across neighbouring pixels to reduce banding artefacts.

Supported formats:

- `rgba8888` — 32 bits per pixel, full precision. Default; no dithering.
- `rgb888` — 24 bits per pixel, no alpha channel.
- `rgb565` — 16 bits per pixel. 5-bit red, 6-bit green, 5-bit blue. No alpha.
- `rgba4444` — 16 bits per pixel. 4 bits per channel.
- `rgba5551` — 16 bits per pixel. 5-bit RGB, 1-bit alpha (threshold at 128).
- `alpha8` — 8 bits per pixel, alpha channel only; RGB set to zero.

## Usage

### CLI Flags

```
--pixel-format <FORMAT>
```

Default: `rgba8888`.

```
fastpack pack sprites/ --output out/ --pixel-format rgb565
fastpack pack sprites/ --output out/ --pixel-format rgba4444
```

Combine with `--texture-format` to control both the pixel encoding and the container:

```
fastpack pack sprites/ --output out/ --pixel-format rgb565 --texture-format jpeg
```

### `.fpsheet` Fields

Pixel format is not currently exposed in the project file. Set it on the command line.

## Examples

Pack a UI atlas to RGBA4444 for a mobile target:

```
fastpack pack ui/ --output out/ --pixel-format rgba4444
```

Pack to RGB565 for a sprite set with no transparency:

```
fastpack pack backgrounds/ --output out/ --pixel-format rgb565
```

Pack an alpha mask sheet:

```
fastpack pack masks/ --output out/ --pixel-format alpha8
```

## Technical Notes

FastPack quantizes in scan order (left to right, top to bottom). The error from each pixel is diffused to four neighbours using the standard Floyd-Steinberg weights:

```
          [7/16]
[3/16] [5/16] [1/16]
```

Quantization uses the replication formula to expand reduced values back to 8 bits. For 5-bit red: `(r5 << 3) | (r5 >> 2)`. This gives the maximum and minimum correct values (255 and 0) and distributes intermediate values evenly.

The dithering step runs after atlas composition and before the compressor. The output image is always RGBA8888 internally; the quantization reduces the number of unique colour values, which can improve compression ratios for PNG and JPEG even though the output stays 8 bits per channel.

`rgba5551` thresholds alpha at 128: pixels at or above 128 become fully opaque (255); pixels below become fully transparent (0). Dithering is applied to the alpha channel, so pixels near the threshold may produce a mix of opaque and transparent results.

`rgb888` and `alpha8` do not dither because they do not reduce bit depth from the source channels (RGB888 keeps 8 bits, Alpha8 keeps the 8-bit alpha exactly).

## TexturePacker Compatibility

TexturePacker exposes pixel format as a per-platform preset option. The format strings (RGBA8888, RGB565, RGBA4444, RGBA5551, Alpha8) match TexturePacker's naming conventions.
