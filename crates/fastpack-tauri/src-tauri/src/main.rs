//! Tauri app entry point. Launches the FastPack GUI window, or runs the CLI
//! when invoked with a subcommand (e.g. `FastPack.AppImage pack sprites/`).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if fastpack::is_cli_invocation(std::env::args_os().skip(1)) {
        if let Err(e) = fastpack::run() {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
        return;
    }

    fastpack_tauri::run(None).expect("error while running FastPack");
}
