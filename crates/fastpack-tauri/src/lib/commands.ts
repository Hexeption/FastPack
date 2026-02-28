/** Tauri invoke wrappers for all backend commands. */

import { invoke } from "@tauri-apps/api/core";
import type { Preferences, Project, ReleaseInfo } from "../types";

// Project lifecycle

/** Loads saved preferences from disk. */
export function getPreferences() {
	return invoke<Preferences>("get_preferences");
}

/** Loads the current in-memory project from the backend. */
export function getProject() {
	return invoke<Project>("get_project");
}

/** Creates a new project with default settings. */
export function newProject() {
	return invoke<Project>("new_project");
}

/** Opens a .fpsheet project file from the given path. */
export function openProject(path: string) {
	return invoke<Project>("open_project", { path });
}

/** Saves the current project to the given path. */
export function saveProject(path: string) {
	return invoke<void>("save_project", { path });
}

/** Pushes a modified project config to the backend without saving to disk. */
export function updateProject(project: Project) {
	return invoke<void>("update_project", { project });
}

/** Adds a sprite source directory to the project. */
export function addSource(path: string) {
	return invoke<Project>("add_source", { path });
}

/** Removes a source by index. */
export function removeSource(index: number) {
	return invoke<Project>("remove_source", { index });
}

/** Processes dropped file/folder paths. Returns the updated project and whether it changed. */
export function handleDrop(paths: string[]) {
	return invoke<{
		project: Project;
		project_path: string | null;
		dirty: boolean;
	}>("handle_drop", { paths });
}

// Packing

/** Runs the sprite packer. Results arrive via pack:finished or pack:failed events. */
export function pack() {
	return invoke<void>("pack");
}

/** Publishes packed sheets to the output directory. */
export function publish() {
	return invoke<void>("publish");
}

/** Starts file system watch on project sources. Repacks automatically on changes. */
export function startWatch() {
	return invoke<void>("start_watch");
}

/** Stops the file system watcher. */
export function stopWatch() {
	return invoke<void>("stop_watch");
}

// Dialogs

/** Opens a native file picker filtered to .fpsheet files. Returns the selected path or null. */
export function openFileDialog() {
	return invoke<string | null>("open_file_dialog");
}

/** Opens a native folder picker. Optionally starts at the given path. */
export function openFolderDialog(startingPath?: string | null) {
	return invoke<string | null>("open_folder_dialog", {
		startingPath: startingPath ?? null,
	});
}

/** Opens a native save dialog with the given default file name. */
export function saveFileDialog(defaultName: string) {
	return invoke<string | null>("save_file_dialog", { defaultName });
}

/** Opens the app config directory in the system file manager. */
export function openConfigFolder() {
	return invoke<void>("open_config_folder");
}

// Preferences

/** Saves preferences to disk. */
export function savePreferences(prefs: Preferences) {
	return invoke<void>("save_preferences", { prefs });
}

// Updates

/** Checks for a newer release. Returns release info or null if up to date. */
export function checkForUpdate() {
	return invoke<ReleaseInfo | null>("check_for_update");
}

/** Downloads an update from the given asset URL. */
export function downloadUpdate(assetUrl: string) {
	return invoke<void>("download_update", { assetUrl });
}

/** Replaces the running binary with the downloaded update. */
export function applyUpdate() {
	return invoke<void>("apply_update");
}

// CLI

/** Installs the fastpack CLI to the user's PATH. */
export function installCli() {
	return invoke<void>("install_cli");
}

/** Returns true if the CLI is already installed. */
export function checkCliInstalled() {
	return invoke<boolean>("check_cli_installed");
}
