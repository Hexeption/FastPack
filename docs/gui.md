# GUI Reference

FastPack opens as a native desktop window when launched with no subcommand. All packing settings, sprite management, and output configuration live in the GUI without needing a .fpsheet file.

## Launching

```
fastpack                    # open the GUI with an empty project
fastpack gui                # same
fastpack gui project.fpsheet  # open the GUI with the project preloaded
```

## Window Layout

The window is divided into five panels.

**Menu bar** — top strip with File and Atlas menus.

**Toolbar** — below the menu bar. Has an Export button, an Add Sprites button, a Preview Animation button (enabled when two or more sprites are selected), source count indicator, and a Light/Dark theme toggle.

**Sprite list** (left panel) — shows source directories at the top and packed frame list below. Resizable by dragging its border.

**Settings** (right panel) — all packing configuration. Resizable.

**Atlas preview** (center) — the composited atlas texture. Updates automatically when any layout, sprite, or algorithm setting changes.

**Output log** (bottom panel) — timestamped messages from pack operations. Resizable.

## Sprite List Panel

The top section lists configured source directories. The Add and Remove buttons manage entries. The Add Sprites toolbar button and the File > Add Sprites… menu item both open a folder picker.

The bottom section shows the packed frames from the most recent pack. Click a frame to select it. The selection highlights the corresponding region in the atlas preview and opens a detail view with:

- Frame and source coordinates.
- Nine-patch editor: checkbox to enable, four inputs for top/right/bottom/left border widths, preview showing the boundary lines over the sprite.
- Pivot editor: checkbox for a custom pivot, X/Y sliders scaled 0.0–1.0, interactive preview where dragging the crosshair updates both sliders.
- Add Override button to persist the nine-patch and pivot values into the project.

Frames marked as aliases show the name of the canonical sprite.

## Atlas Preview Panel

The atlas texture fills the center panel. Scroll to zoom (0.05× to 64×). Click and drag to pan. Double-click to reset zoom and pan. The current zoom percentage and atlas dimensions are shown in the top-left corner.

A golden rectangle outlines the selected frame during a selection.

The bottom-right corner shows sprite count, alias count, and overflow count from the last pack.

A blue overlay with "Drop folders or .fpsheet here" appears when files are dragged over the window.

## Settings Panel

The settings panel has four collapsible sections.

**Texture** controls output path (text field + Browse button), name prefix, texture format (PNG/JPEG/WebP), pixel format, data format (JSON Hash/JSON Array/Phaser 3/PixiJS), quality, premultiply alpha, and texture path prefix. These settings affect export output only and do not trigger a repack.

**Layout** controls max width/height, optional fixed width/height, size constraint (Any/Power of 2/Multiple of 4/Word aligned), force square toggle, allow rotation toggle, border padding, shape padding, algorithm (Grid/Basic/MaxRects/Polygon), MaxRects heuristic, and pack mode (Fast/Good/Best).

**Sprites** controls trim mode (None/Trim/Crop/CropKeepPos/Polygon), trim margin, trim threshold, extrude, common divisors, and alias detection toggle.

**Variants** lists scale variants. Each variant has a scale factor, scale mode (Smooth/Fast/Scale2x/Scale3x/HQ2x — pixel art modes are planned but not yet implemented), and a filename suffix. Add and remove variants with the + and − buttons.

Any change to a layout, sprite, or algorithm setting repacks the atlas automatically and updates the preview. Any change to a setting marks the project as dirty (asterisk in the window title).

## Auto-Repack

Changing any layout, sprite, or algorithm setting triggers an immediate repack. The atlas preview updates in place without needing to click a button. Output-only settings (texture format, data format, quality, output directory, name) do not repack.

## Exporting

The Export button in the toolbar and the Atlas > Export menu item write the atlas to disk. Set an output directory in the Texture settings first. Export compresses the atlas image using the configured texture format (PNG/JPEG/WebP) and writes a data file in the selected data format alongside it. File sizes are logged in the output panel.

## Output Log Panel

Messages appear with a timestamp prefix. Info messages are grey, warnings are orange, errors are red. The log scrolls automatically to the latest entry. A Clear button empties it.

## Keyboard Shortcuts

```
Ctrl+N   New project
Ctrl+O   Open project file
Ctrl+S   Save project file
Ctrl+P   Export atlas to disk
P        Open/close animation preview (when two or more sprites are selected)
```

Shortcuts work both through the menus and directly while any panel has focus. All shortcuts can be rebound in Edit → Preferences → Keybinds. See `docs/features/keybinds.md`.

## Drag and Drop

Dropping a .fpsheet file onto the window opens it as the current project.

Dropping a folder adds it as a source directory and triggers an automatic pack.

Dropping an image file adds its parent folder as a source directory.

## Project Files

Open and save with File > Open/Save or the corresponding keyboard shortcut. The window title shows the project name with an asterisk when there are unsaved changes.

The project file is TOML with a `.fpsheet` extension. See `docs/project-file.md` for the full schema.

## Theme

The toolbar's Light/Dark button toggles between the custom dark theme (default) and a standard light theme. The choice persists in app state but is not saved to the project file.

## TexturePacker Compatibility

The GUI layout and settings mirror TexturePacker's settings panel. All core layout fields (max size, padding, rotation, size constraints, trim mode) correspond directly to TexturePacker options. The data format selector covers the same output targets. Settings are stored in the project file rather than a separate history file.
