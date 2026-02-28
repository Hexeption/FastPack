import { invoke } from "@tauri-apps/api/core";
import type { Preferences, Project, ReleaseInfo } from "../types";

// Project lifecycle
export function getPreferences() {
	return invoke<Preferences>("get_preferences");
}

export function getProject() {
	return invoke<Project>("get_project");
}

export function newProject() {
	return invoke<Project>("new_project");
}

export function openProject(path: string) {
	return invoke<Project>("open_project", { path });
}

export function saveProject(path: string) {
	return invoke<void>("save_project", { path });
}

export function updateProject(project: Project) {
	return invoke<void>("update_project", { project });
}

export function addSource(path: string) {
	return invoke<Project>("add_source", { path });
}

export function removeSource(index: number) {
	return invoke<Project>("remove_source", { index });
}

export function handleDrop(paths: string[]) {
	return invoke<{
		project: Project;
		project_path: string | null;
		dirty: boolean;
	}>("handle_drop", { paths });
}

// Packing
export function pack() {
	return invoke<void>("pack");
}

export function publish() {
	return invoke<void>("publish");
}

export function startWatch() {
	return invoke<void>("start_watch");
}

export function stopWatch() {
	return invoke<void>("stop_watch");
}

// Dialogs
export function openFileDialog() {
	return invoke<string | null>("open_file_dialog");
}

export function openFolderDialog(startingPath?: string | null) {
	return invoke<string | null>("open_folder_dialog", {
		startingPath: startingPath ?? null,
	});
}

export function saveFileDialog(defaultName: string) {
	return invoke<string | null>("save_file_dialog", { defaultName });
}

export function openConfigFolder() {
	return invoke<void>("open_config_folder");
}

// Preferences
export function savePreferences(prefs: Preferences) {
	return invoke<void>("save_preferences", { prefs });
}

// Updates
export function checkForUpdate() {
	return invoke<ReleaseInfo | null>("check_for_update");
}

export function downloadUpdate(assetUrl: string) {
	return invoke<void>("download_update", { assetUrl });
}

export function applyUpdate() {
	return invoke<void>("apply_update");
}

// CLI
export function installCli() {
	return invoke<void>("install_cli");
}

export function checkCliInstalled() {
	return invoke<boolean>("check_cli_installed");
}
