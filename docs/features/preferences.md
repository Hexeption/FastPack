# Preferences

The Preferences window lets you set app-wide options that persist across projects.
Open it from Edit → Preferences…

## General tab

Sets the UI language. FastPack ships with translations for English, French, Spanish, German, Italian, Portuguese, Japanese, Simplified Chinese, and Korean. Changing the language takes effect immediately.

## Defaults tab

Every setting from the right-side panel is available here: Texture, Layout, Sprites, and Variants.

Changes are applied immediately and saved to the preferences file. When you create a new project (File → New), FastPack starts with these defaults instead of the built-in ones.

This is useful if you always target a specific data format, atlas size, or trim mode. Set those once in Preferences and every new project begins configured the way you need.

The preferences file is stored at:

- Windows: `%APPDATA%\FastPack\prefs.toml`
- macOS: `~/Library/Application Support/FastPack/prefs.toml`
- Linux: `~/.config/FastPack/prefs.toml`

It is plain TOML. You can edit it by hand or delete it to reset all defaults.

## Keybinds tab

All five keyboard shortcuts can be rebound here. Click **Change** next to an action, then press the new key combination. Modifier keys (Ctrl, Alt, Shift) are captured as part of the binding. Press **Escape** to cancel without making a change. **Reset to defaults** restores all five shortcuts at once.

Supported keys: A–Z, 0–9, F1–F12, Enter, Space, Tab, Delete, Backspace, Home, End, Insert.

Bindings are stored in the `[keybinds]` section of the preferences file. See `docs/features/keybinds.md` for the full list of defaults.

## Updates tab

Shows the current installed version and the latest release on GitHub.

FastPack checks for updates at startup if "Check automatically on startup" is enabled. You can also click "Check for Updates" at any time.

When a newer version is available, release notes appear and a "Download and Install" button downloads the new binary. After download completes, "Restart and Update" replaces the current executable and restarts the app.

The update mechanism downloads the platform binary from the GitHub release assets and performs an in-place replacement:

- On Windows a small batch script handles the swap after the process exits.
- On macOS and Linux the binary is copied over and the new version is launched.

If the release does not include a binary for the current platform, the check silently reports up to date rather than showing an error.
