use std::path::Path;
use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fastpack"))
}

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/sprites")
}

// Help / version

#[test]
fn help_flag_exits_zero() {
    let status = binary()
        .arg("--help")
        .status()
        .expect("failed to start fastpack");
    assert!(status.success(), "--help should exit 0");
}

#[test]
fn pack_subcommand_help_exits_zero() {
    let status = binary()
        .args(["pack", "--help"])
        .status()
        .expect("failed to start fastpack");
    assert!(status.success(), "pack --help should exit 0");
}

// pack subcommand — success paths

#[test]
fn pack_fixture_sprites_produces_output_files() {
    let out = tempfile::tempdir().expect("tempdir");
    let status = binary()
        .args([
            "pack",
            fixtures_dir().to_str().unwrap(),
            "--output",
            out.path().to_str().unwrap(),
            "--name",
            "test_atlas",
        ])
        .status()
        .expect("failed to start fastpack");
    assert!(status.success(), "pack should exit 0 on valid input");

    let texture = out.path().join("test_atlas.png");
    let data = out.path().join("test_atlas.json");
    assert!(texture.exists(), "texture file should exist: {texture:?}");
    assert!(data.exists(), "data file should exist: {data:?}");
}

#[test]
fn pack_json_data_file_has_frames_key() {
    let out = tempfile::tempdir().expect("tempdir");
    binary()
        .args([
            "pack",
            fixtures_dir().to_str().unwrap(),
            "--output",
            out.path().to_str().unwrap(),
            "--name",
            "atlas",
        ])
        .status()
        .expect("failed to start fastpack");

    let content = std::fs::read_to_string(out.path().join("atlas.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        json.get("frames").is_some(),
        "output JSON should have a 'frames' key"
    );
}

#[test]
fn pack_with_phaser3_format_produces_textures_key() {
    let out = tempfile::tempdir().expect("tempdir");
    binary()
        .args([
            "pack",
            fixtures_dir().to_str().unwrap(),
            "--output",
            out.path().to_str().unwrap(),
            "--name",
            "atlas",
            "--data-format",
            "phaser3",
        ])
        .status()
        .expect("failed to start fastpack");

    let content = std::fs::read_to_string(out.path().join("atlas.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        json.get("textures").is_some(),
        "phaser3 output should have a 'textures' key"
    );
}

// pack subcommand — error paths

#[test]
fn pack_no_inputs_exits_nonzero() {
    let out = tempfile::tempdir().expect("tempdir");
    let status = binary()
        .args(["pack", "--output", out.path().to_str().unwrap()])
        .status()
        .expect("failed to start fastpack");
    assert!(
        !status.success(),
        "pack with no inputs should exit non-zero"
    );
}

#[test]
fn pack_nonexistent_input_exits_nonzero() {
    let out = tempfile::tempdir().expect("tempdir");
    let status = binary()
        .args([
            "pack",
            "/nonexistent/path/that/does/not/exist",
            "--output",
            out.path().to_str().unwrap(),
        ])
        .status()
        .expect("failed to start fastpack");
    assert!(
        !status.success(),
        "pack with nonexistent input should exit non-zero"
    );
}
