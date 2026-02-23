# CLI Reference

`fastpack` is the headless command-line interface. Running it without arguments opens the GUI (Phase 5). Use subcommands for headless operation.

## Subcommands

```
fastpack pack   <inputs...> [flags]   Pack sprites into a texture atlas
fastpack --help                       Show top-level help
fastpack <subcommand> --help          Show subcommand help
```

## fastpack pack

Pack one or more directories (or individual files) of sprites into a single texture atlas.

```
fastpack pack <INPUT>... [OPTIONS]
```

### Arguments

`INPUT` — One or more paths. Directories are searched recursively for image files (`png`, `jpg`, `jpeg`, `bmp`, `tga`, `webp`, `tiff`, `gif`). Files are added directly.

### Options

`-o, --output <dir>` (default: `output`) — Output directory for the texture and data files. Created if it does not exist.

`--name <basename>` (default: `atlas`) — Base filename for output files without extension. Produces `<name>.png` and `<name>.json`.

`--max-width <n>` (default: `4096`) — Maximum atlas width in pixels.

`--max-height <n>` (default: `4096`) — Maximum atlas height in pixels.

`--pack-mode <fast|good|best>` (default: `good`) — Compression effort. `fast` skips oxipng. `good` runs oxipng at preset 3. `best` runs oxipng at preset 6.

### Example

```
fastpack pack sprites/ --output build/atlas --name ui --max-width 2048
```

Writes `build/atlas/ui.png` and `build/atlas/ui.json`.

Multiple input directories:

```
fastpack pack sprites/ui sprites/world --output out --name game
```

### Sprite IDs

Each frame's ID is the file path relative to the input directory, without extension, with forward slashes as the separator.

For a file at `sprites/player/idle.png` loaded from root `sprites/`, the ID is `player/idle`.

When individual files are passed as inputs, the ID is the filename without extension (no directory component).

## Output Files

`<name>.json` — Atlas metadata in JSON Hash format. Contains `frames` (one entry per sprite) and a `meta` block with atlas dimensions, image filename, pixel format, and scale.

`<name>.png` — Lossless PNG texture. Pixel format is RGBA8888.

## Technical Notes

Phase 1 uses the MaxRects algorithm with `BestShortSideFit` heuristic, `allow_rotation = true`, and `shape_padding = 2`. Sprites are trimmed with `TrimMode::Trim` and alpha threshold 1. These settings are not yet configurable via flags; full flag coverage is planned for Phase 2.

Exit code is 0 on success. Exit code is non-zero when no images are found, all images fail to load, or the packer fails.

If overflow occurs (sprites that could not fit), a warning is printed to stderr. The atlas is still written with the sprites that did fit.

## TexturePacker Compatibility

`fastpack pack` accepts a subset of the flags TexturePacker CLI exposes. Full compatibility mapping (including `--format`, `--data`, `--sheet`) is planned for Phase 6.
