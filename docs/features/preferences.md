# Preferences

The Preferences window lets you set app-wide options that persist across projects.
Open it from Edit → Preferences…

## Defaults tab

Every setting from the right-side panel is available here: Texture, Layout, Sprites, and Variants.

Changes are applied immediately and saved to the preferences file. When you create a new project (File → New), FastPack starts with these defaults instead of the built-in ones.

This is useful if you always target a specific data format, atlas size, or trim mode. Set those once in Preferences and every new project begins configured the way you need.

The preferences file is stored at:

- Windows: `%APPDATA%\FastPack\prefs.toml`
- macOS: `~/Library/Application Support/FastPack/prefs.toml`
- Linux: `~/.config/FastPack/prefs.toml`

It is plain TOML. You can edit it by hand or delete it to reset all defaults.

## Updates tab

Shows the current installed version and the latest release on GitHub.

FastPack checks for updates at startup if "Check automatically on startup" is enabled. You can also click "Check for Updates" at any time.

When a newer version is available, release notes appear and a "Download and Install" button downloads the new binary. After download completes, "Restart and Update" replaces the current executable and restarts the app.

The update mechanism downloads the platform binary from the GitHub release assets and performs an in-place replacement:

- On Windows a small batch script handles the swap after the process exits.
- On macOS and Linux the binary is copied over and the new version is launched.

If the release does not include a binary for the current platform, the check silently reports up to date rather than showing an error.
