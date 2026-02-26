// Tauri app entry point.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    fastpack_tauri::run(None).expect("error while running FastPack");
}
