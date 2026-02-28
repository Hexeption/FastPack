# Sprite Sheet Splitter

The `split` subcommand is the inverse of packing. It reads an atlas image and its data file, then writes each sprite back out as a separate PNG file. Use it to extract sprites from a legacy atlas, inspect packed output, or feed sprites into an editing tool.

## Usage

```
fastpack split <atlas.png> [--data <atlas.json>] [--output <dir>]
```

If `--data` is omitted, FastPack looks for a JSON file with the same base name as the atlas image in the same directory.

If `--output` is omitted, sprites are written to a `split/` subdirectory next to the atlas.

### CLI Flags

```
--data FILE    data file to read frame rects from (default: <atlas>.json)
--output DIR   directory for extracted sprites (default: split/ next to atlas)
```

## Examples

```
fastpack split output/atlas.png
# writes to: output/split/player/idle.png, output/split/ui/button.png, ...

fastpack split output/atlas.png --data output/atlas.json --output extracted/
```

Extracted sprites are placed in subdirectories matching their sprite ID path:

```
extracted/
  player/
    idle.png
    walk_01.png
  ui/
    button.png
    panel.png
```

## Technical Notes

Sprites are extracted using the `frame` rect from the data file, which points at the trimmed region in the atlas. Each extracted PNG has the dimensions `frame.w × frame.h` — the trimmed size, not the original source image size.

To reconstruct the full original dimensions including transparent padding, you would composite the extracted sprite at offset `spriteSourceSize.x`, `spriteSourceSize.y` inside a canvas of `sourceSize.w × sourceSize.h`. The splitter extracts the packed region as-is and does not reconstruct transparent gutters.

Alias sprites (those with an `aliasOf` field) are still written as separate files; each gets the pixels of the atlas frame they point to.

## TexturePacker Compatibility

TexturePacker includes a similar "Unpack Sprites" command. Both tools extract sprites from the `frame` rect; the transparent-gutter reconstruction behavior described above applies to both.
