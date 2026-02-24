# Animation Preview

The animation preview window plays a sequence of selected sprites as a flipbook animation. It is useful for checking frame timing, verifying that animation frames are correctly packed, and spot-checking sprite alignment before export.

## Usage

Pack your atlas, then select two or more sprites in the sprite list. Sprites can be selected with:

- **Click** — select one sprite, clear others
- **Ctrl+click** — toggle a sprite in or out of the selection
- **Shift+click** — extend the selection from the last plain-clicked sprite to the clicked sprite
- **Shift+Ctrl+click** — same range extension, but adds to the existing selection instead of replacing it

Once two or more sprites are selected, open the animation preview with:

- The **Preview Animation [P]** button at the bottom of the sprite list, or
- The **P** key (no modifier)

Pressing P again closes the window.

## Controls

**◀ / ▶|** — Step one frame backward or forward. Pauses playback.

**▶ / ⏸** — Toggle playback on and off.

**FPS slider** — Set the playback rate from 1 to 60 frames per second.

**Frame counter** — Shows the current frame number and total frame count (e.g. `3 / 12`).

**Loop** — When checked, playback wraps back to the first frame after the last. When unchecked, playback stops on the last frame.

**Scroll wheel** — Zoom the canvas in or out.

**Drag** — Pan the canvas.

**Double-click** — Reset zoom and pan to the default (1× centred).

## Technical Notes

Frame selection order determines playback order. Frames play in the order they were added to the selection (click order), not alphabetical order. To control animation order precisely, click sprites one at a time in the intended sequence.

The canvas is a fixed viewport. It does not resize when switching between frames of different sizes. Each frame is drawn centred at the current zoom and pan, so size differences between frames are visible as the image changing position or scale within the stable canvas.

Frame advance uses the frame's delta time (`unstable_dt`) accumulated against the selected FPS. If a frame takes longer than expected (e.g. the window is hidden or the process is paused), accumulated time is consumed in a loop to catch up rather than skipping frames entirely.

Zoom and pan persist while the window is open. They reset to defaults when the window is reopened from the button or P key.

Alias sprites (detected duplicates) appear in the list and can be included in an animation selection. They share the same atlas region as their canonical sprite, so they render identically.
