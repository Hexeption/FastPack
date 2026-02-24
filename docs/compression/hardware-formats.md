# Hardware Texture Formats

FastPack can write atlas textures in several formats beyond PNG. Use `--texture-format` to select the output format. The default is `png`.

## Overview

GPU-native compressed formats (DXT1, DXT5) reduce GPU memory footprint and improve texture fetch bandwidth. Unlike PNG, which is decompressed by the CPU before upload, these formats stay compressed in VRAM and are sampled directly by the hardware.

DXT1 and DXT5 are supported natively in FastPack with no external dependencies. Each format produces a DDS (DirectDraw Surface) container file.

Other hardware formats (ETC1/2, PVRTC, ASTC, Basis Universal) require external encoder tools and fall back to PNG output.

## Usage

### CLI Flags

```
--texture-format <FORMAT>
```

Accepted values: `png`, `jpeg`, `webp`, `dxt1`, `dxt5`. Default: `png`.

Example:

```
fastpack pack sprites/ --output out/ --texture-format dxt1
fastpack pack sprites/ --output out/ --texture-format dxt5
```

The output texture file uses the extension matching the format: `.png`, `.jpg`, `.webp`, or `.dds` for DXT1/DXT5.

### `.fpsheet` Fields

The project file does not currently expose `texture_format` as a field. Use the CLI flag when building from a project file.

## Examples

Pack a sprite directory to DXT1 (no alpha, smallest DDS size):

```
fastpack pack sprites/ --output out/ --texture-format dxt1
```

Pack with DXT5 (full alpha channel):

```
fastpack pack sprites/ --output out/ --texture-format dxt5
```

Watch mode works the same way:

```
fastpack watch sprites/ --output out/ --texture-format dxt5
```

## Technical Notes

DXT1 encodes each 4x4 pixel block in 8 bytes (4 bits per pixel). The block has two 16-bit RGB565 endpoints and 16 two-bit indices. FastPack uses a min/max bounding-box endpoint selector. This is fast and produces valid BC1 blocks, though iterative cluster-fit encoders achieve slightly better quality.

DXT1 has no per-pixel alpha. All pixels in the atlas are treated as opaque. Use DXT5 when sprites have soft or partial transparency.

DXT5 encodes each 4x4 block in 16 bytes: an 8-byte BC3 alpha block followed by an 8-byte BC1 colour block. The alpha block stores two endpoint values and six linearly interpolated steps, with three-bit indices for each pixel.

Both formats write a standard DDS file with a 128-byte header (4-byte magic + 124-byte `DDS_HEADER`). The pixel format block uses the FourCC codes `DXT1` and `DXT5` respectively. The format is read by DirectX, Unity, Unreal Engine, and most game-oriented texture loaders.

DDS files are larger on disk than PNG for typical sprite atlases: DXT1 uses a fixed 4 bits/pixel while PNG compresses losslessly. The benefit is runtime memory; a DXT1 texture uses 4 bits/pixel in VRAM versus 32 bits/pixel for an RGBA PNG.

## TexturePacker Compatibility

TexturePacker calls these formats `PVRTC4`, `ETC1`, or `DTX1`/`DTX5` depending on the target platform preset. FastPack uses the same DDS container and FourCC codes, so DDS files produced by FastPack load correctly wherever TexturePacker DDS output is accepted.
