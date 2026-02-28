//! Tauri app entry point. Launches the FastPack GUI window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    fastpack_tauri::run(None).expect("error while running FastPack");
}
