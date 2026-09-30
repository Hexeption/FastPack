//! FastPack CLI binary.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    fastpack::run()
}
