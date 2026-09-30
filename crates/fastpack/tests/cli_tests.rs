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

// sprite naming options

/// Pack `input` with extra flags into a fresh temp dir and return the parsed
/// JSON Hash data file alongside the temp dir (kept alive by the caller).
fn pack_json_hash(input: &Path, extra: &[&str]) -> (tempfile::TempDir, serde_json::Value) {
    let out = tempfile::tempdir().expect("tempdir");
    let mut args = vec![
        "pack",
        input.to_str().unwrap(),
        "--output",
        out.path().to_str().unwrap(),
        "--name",
        "atlas",
        "--data-format",
        "json-hash",
    ];
    args.extend_from_slice(extra);
    let status = binary().args(&args).status().expect("failed to start");
    assert!(status.success(), "pack {extra:?} should exit 0");
    let content = std::fs::read_to_string(out.path().join("atlas.json")).unwrap();
    (out, serde_json::from_str(&content).unwrap())
}

fn frame_names(json: &serde_json::Value) -> Vec<String> {
    let mut names: Vec<String> = json["frames"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    names.sort();
    names
}

#[test]
fn pack_default_names_strip_extension() {
    let (_out, json) = pack_json_hash(&fixtures_dir(), &[]);
    assert_eq!(frame_names(&json), ["blue", "green", "red", "wide"]);
}

#[test]
fn pack_keep_extension_keeps_png_suffix() {
    let (_out, json) = pack_json_hash(&fixtures_dir(), &["--keep-extension"]);
    assert_eq!(
        frame_names(&json),
        ["blue.png", "green.png", "red.png", "wide.png"]
    );
}

#[test]
fn pack_prepend_folder_name_prefixes_source_dir() {
    let (_out, json) = pack_json_hash(&fixtures_dir(), &["--prepend-folder-name"]);
    assert_eq!(
        frame_names(&json),
        [
            "sprites/blue",
            "sprites/green",
            "sprites/red",
            "sprites/wide"
        ]
    );

    let (_out, json) = pack_json_hash(
        &fixtures_dir(),
        &["--prepend-folder-name", "--keep-extension"],
    );
    assert!(json["frames"].get("sprites/wide.png").is_some());
}

#[test]
fn pack_prepend_folder_name_ignores_single_file_inputs() {
    let (_out, json) = pack_json_hash(
        &fixtures_dir().join("red.png"),
        &["--prepend-folder-name", "--keep-extension"],
    );
    assert_eq!(frame_names(&json), ["red.png"]);
}

#[test]
fn pack_keep_extension_alias_refers_to_export_name() {
    let src = tempfile::tempdir().expect("tempdir");
    let sprites = src.path().join("hero");
    std::fs::create_dir_all(&sprites).unwrap();
    std::fs::copy(fixtures_dir().join("red.png"), sprites.join("a.png")).unwrap();
    std::fs::copy(fixtures_dir().join("red.png"), sprites.join("b.png")).unwrap();

    let (_out, json) = pack_json_hash(&sprites, &["--keep-extension", "--prepend-folder-name"]);
    assert_eq!(frame_names(&json), ["hero/a.png", "hero/b.png"]);
    assert_eq!(json["frames"]["hero/b.png"]["aliasOf"], "hero/a.png");
}

#[test]
fn project_overrides_and_excludes_match_ids_with_naming_options() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project = dir.path().join("p.fpsheet");
    let toml = format!(
        r#"excludes = ["red"]

[[sources]]
path = {src:?}
filter = "**/*.png"

[[sprite_overrides]]
id = "blue"
pivot = {{ x = 0.25, y = 0.75 }}

[layout]
max_width = 2048
max_height = 2048
size_constraint = "any_size"
force_square = false
allow_rotation = false
pack_mode = "good"
border_padding = 2
shape_padding = 2

[sprites]
trim_mode = "trim"
trim_threshold = 1
trim_margin = 0
extrude = 0
common_divisor_x = 0
common_divisor_y = 0
detect_aliases = true
keep_extension = true
prepend_folder_name = true

[sprites.default_pivot]
x = 0.0
y = 0.0

[output]
name = "atlas"
directory = "output"
texture_format = "png"
pixel_format = "rgba8888"
premultiply_alpha = false
data_format = "json_hash"
quality = 95
texture_path_prefix = ""
multipack = false

[algorithm]
type = "max_rects"
heuristic = "best_short_side_fit"

[[variants]]
scale = 1.0
suffix = ""
scale_mode = "smooth"
"#,
        src = fixtures_dir().to_str().unwrap()
    );
    std::fs::write(&project, toml).unwrap();

    let out = dir.path().join("out");
    let status = binary()
        .args([
            "pack",
            "--project",
            project.to_str().unwrap(),
            "--output",
            out.to_str().unwrap(),
        ])
        .status()
        .expect("failed to start");
    assert!(status.success());

    let content = std::fs::read_to_string(out.join("atlas.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(
        frame_names(&json),
        ["sprites/blue.png", "sprites/green.png", "sprites/wide.png"]
    );
    let pivot = &json["frames"]["sprites/blue.png"]["pivot"];
    assert_eq!(pivot["x"], 0.25);
    assert_eq!(pivot["y"], 0.75);
}

#[test]
fn split_does_not_double_png_extension() {
    let (out, _json) = pack_json_hash(
        &fixtures_dir(),
        &["--keep-extension", "--prepend-folder-name"],
    );
    let split_dir = out.path().join("split");
    let status = binary()
        .args([
            "split",
            out.path().join("atlas.png").to_str().unwrap(),
            out.path().join("atlas.json").to_str().unwrap(),
            "--output-dir",
            split_dir.to_str().unwrap(),
        ])
        .status()
        .expect("failed to start");
    assert!(status.success());
    assert!(split_dir.join("sprites").join("blue.png").exists());
    assert!(!split_dir.join("sprites").join("blue.png.png").exists());
}
