# Tauri GUI

FastPack uses [Tauri 2](https://tauri.app) for its desktop GUI. The frontend is React + TypeScript, served by Vite. The Rust backend exposes packing, project management, and file operations as Tauri commands.

## Setup

Install Node dependencies before running the app:

```
cd crates/fastpack-tauri
npm install
```

Tauri CLI is included as a dev dependency so no global install is needed.

## Running in development

```
cd crates/fastpack-tauri
npm run tauri dev
```

This starts Vite's dev server on port 5173 and the Tauri window connects to it. Hot module replacement works for the frontend; Rust changes restart the backend automatically.

## Building a release

```
cd crates/fastpack-tauri
npm run tauri build
```

The packaged app (`.dmg`, `.msi`, or `.AppImage`) lands in `src-tauri/target/release/bundle/`.

## Icons

Drop your icon source (PNG or SVG, at least 1024×1024) at `assets/icon.png` and run:

```
cd crates/fastpack-tauri
npx tauri icon ../../assets/icon.png
```

This generates all required sizes into `src-tauri/icons/`.

## Architecture

```
crates/fastpack-tauri/
  src/                    React + TypeScript frontend
    components/           UI panels and dialogs
    hooks/usePack.ts      Listens for pack:started/finished/failed events
    store.ts              Zustand global state
    types.ts              TypeScript mirrors of Rust types
  src-tauri/              Tauri Rust backend (workspace member)
    src/
      commands/           Tauri command handlers
      state.rs            Shared app state (Mutex<TauriState>)
      worker.rs           Pack pipeline (same logic as old egui backend)
      preferences.rs      User preferences (load/save prefs.toml)
      updater.rs          GitHub release checking and auto-update
```

## Commands

The frontend calls these via `invoke()` from `@tauri-apps/api/core`.

`new_project` — Reset to a blank project. Returns the new `Project`.

`open_project(path)` — Load a `.fpsheet` file. Returns `Project` or error.

`save_project(path)` — Save the current project to `path`.

`get_project` — Return the current `Project`.

`update_project(project)` — Replace the current project config.

`add_source(path)` — Add a sprite directory. Returns updated `Project`.

`remove_source(index)` — Remove source by index. Returns updated `Project`.

`pack` — Trigger a pack run. Progress comes back as events (see below).

`start_watch` — Begin watching source directories for changes.

`stop_watch` — Stop the file watcher.

`open_folder_dialog` — Open a native folder picker. Returns `string | null`.

`open_file_dialog` — Open a native file picker (`.fpsheet`). Returns `string | null`.

`save_file_dialog(defaultName)` — Open a native save dialog. Returns `string | null`.

`get_preferences` — Return current `Preferences`.

`save_preferences(prefs)` — Persist preferences to disk.

`check_for_update` — Query GitHub releases API. Returns `ReleaseInfo | null`.

`download_update(url)` — Download a release asset. Returns local path.

`apply_update(path)` — Launch installer and exit.

## Events

The Rust backend emits these events to the frontend window:

`pack:started` — Pack has begun; no payload.

`pack:finished` — Pack succeeded. Payload: `{ sprite_count, alias_count, overflow_count, sheets, log }`.

`pack:failed` — Pack failed. Payload: `{ error }`.

## Preferences

Preferences are stored at:

- macOS: `~/Library/Application Support/FastPack/prefs.toml`
- Windows: `%APPDATA%\FastPack\prefs.toml`
- Linux: `~/.config/FastPack/prefs.toml`

Fields: `dark_mode` (bool), `auto_check_updates` (bool), `language` (string), `ui_scale` (float).

## TexturePacker compatibility

The Tauri GUI exposes the same packing settings and export formats as the CLI and the old egui GUI. Project files (`.fpsheet`) created with any interface are interchangeable.
