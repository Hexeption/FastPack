# Architecture

## Pipeline

FastPack processes sprites in nine sequential stages. Each stage operates on the output of the previous one.

**Collect** — Walk source directories using glob patterns from the `.fpsheet`. Produces a list of `(PathBuf, sprite_id)` pairs. The sprite ID is the file path relative to the source root, stripped of its extension (e.g. `sprites/ui/button.png` → `ui/button`).

**Load** — Load each source file and normalize it to straight-alpha RGBA8. PNG, JPEG, BMP, TGA, WebP, and TIFF are handled by the `image` crate. SVG files are rasterized at their natural viewport size using `resvg` (requires the `svg` feature). PSD files are flattened to a single RGBA layer using `psd` (requires the `psd` feature). Loading runs in parallel via `rayon`. A 64-bit FxHash of the raw pixel bytes is computed here and stored on the `Sprite` for later alias detection.

**Pre-process** — Apply per-sprite transformations in parallel: trim transparent borders, compute convex hull polygons (when `TrimMode::Polygon` is active), and extrude border pixels. Results update the `Sprite`'s `trim_rect` and `image` fields.

**Dedup** — Group sprites by `content_hash`. Within each group, pixel-compare candidates to confirm they are identical. Duplicate sprites get their `alias_of` field set to the canonical sprite ID and are excluded from packing. They still appear in the exported data file pointing at the canonical frame.

**Scale** — When scale variants are configured, fork the sprite list for each variant. Each variant re-scales the images and re-trims. Variants run in parallel; each produces its own independent pack.

**Pack** — Sort sprites by area descending and feed them to the selected `Packer` implementation (MaxRects, Grid, or Basic). The packer returns a `PackOutput` with a `Placement` for every sprite and an `atlas_size`. Sprites that do not fit are returned as `overflow` and trigger a new atlas in multipack mode.

**Compose** — For each atlas, blit placed sprites onto a blank `image::RgbaImage` at their assigned coordinates, rotating 90° clockwise where `Placement::rotated` is `true`.

**Compress** — Pass the composed atlas image to the configured `Compressor`. The compressor writes the texture file and returns the byte count. PNG, JPEG, and WebP are handled in `fastpack-compress`. Hardware formats (DXT, ASTC, ETC, PVRTC, Basis) are handled by optional backends in the same crate.

**Export** — Serialize frame metadata using the configured `Exporter` and write the data file. The exporter receives the full `PackedAtlas` including frame rects, trim offsets, source sizes, rotation flags, and any nine-patch or pivot metadata.

## Crate Layout

`fastpack-core` — algorithms, types, and the imaging pipeline. No CLI or GUI dependencies.

`fastpack-formats` — export format writers. Depends on `fastpack-core`. Each format implements the `Exporter` trait.

`fastpack-compress` — compression backends. Depends on `fastpack-core`. Each backend implements the `Compressor` trait.

`fastpack-gui` — primary interface. An `eframe` native desktop application. Depends on all three library crates.

`fastpack` — CLI binary. Orchestrates the nine-step pipeline. Also launches the GUI when requested.

## Key Types

`Sprite` — a loaded source image plus metadata. Created by `imaging::loader::load`. Mutated by the pre-process stage. Never cloned; moved through the pipeline.

`Placement` — the position and rotation of a single sprite within an atlas, produced by `Packer::pack`.

`PackedAtlas` — the composed atlas image plus the full list of `AtlasFrame` records. Does not implement `Clone` because it owns a `DynamicImage`.

`PackerConfig` — the complete configuration for a single pack run, deserialized from `.fpsheet`. Passed by reference to every pipeline stage.

## Error Handling

Each crate defines its own typed error enum using `thiserror`. `fastpack-core` exports `CoreError`. Errors that cross crate boundaries are wrapped with `#[from]` or mapped explicitly. The pipeline layer in `fastpack` collects per-sprite errors and reports them without aborting the entire run.

## Source Format Notes

SVG rasterization uses straight alpha. `tiny_skia` (used by `resvg`) produces premultiplied RGBA internally; the loader converts to straight alpha before storing the pixel data.

PSD support flattens all visible layers. Individual layer access is not exposed.

Files with unrecognised extensions fall through to `image::open`, which probes by magic bytes. Unsupported formats return `CoreError::ImageLoad`.
