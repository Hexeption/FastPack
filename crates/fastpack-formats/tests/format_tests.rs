use fastpack_core::types::{
    atlas::{AtlasFrame, PackedAtlas},
    rect::{Point, Rect, Size, SourceRect},
    sprite::NinePatch,
};
use fastpack_formats::{
    exporter::{ExportInput, Exporter},
    formats::{json_hash::JsonHashExporter, phaser3::Phaser3Exporter, pixijs::PixiJsExporter},
};
use serde_json::Value;

// --- Helpers ---

fn make_frame(id: &str, x: u32, y: u32, w: u32, h: u32) -> AtlasFrame {
    AtlasFrame {
        id: id.to_string(),
        frame: Rect { x, y, w, h },
        rotated: false,
        trimmed: false,
        sprite_source_size: SourceRect { x: 0, y: 0, w, h },
        source_size: Size { w, h },
        polygon: None,
        nine_patch: None,
        pivot: None,
        alias_of: None,
    }
}

fn make_atlas(frames: Vec<AtlasFrame>) -> PackedAtlas {
    PackedAtlas {
        frames,
        size: Size { w: 256, h: 128 },
        image: None,
        name: "atlas".to_string(),
        scale: 1.0,
    }
}

fn export_input(atlas: &PackedAtlas) -> ExportInput<'_> {
    ExportInput {
        atlas,
        texture_filename: "atlas.png".to_string(),
        pixel_format: "RGBA8888".to_string(),
    }
}

// JsonHashExporter

#[test]
fn json_hash_output_has_frames_and_meta_keys() {
    let atlas = make_atlas(vec![make_frame("hero", 0, 0, 64, 64)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(json.get("frames").is_some(), "missing 'frames' key");
    assert!(json.get("meta").is_some(), "missing 'meta' key");
}

#[test]
fn json_hash_frame_id_is_top_level_key() {
    let atlas = make_atlas(vec![make_frame("ui/button", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(
        json["frames"].get("ui/button").is_some(),
        "sprite id should be a key in 'frames'"
    );
}

#[test]
fn json_hash_multiple_frames_all_present() {
    let atlas = make_atlas(vec![
        make_frame("a", 0, 0, 32, 32),
        make_frame("b", 32, 0, 48, 48),
        make_frame("c", 80, 0, 16, 16),
    ]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(json["frames"].get("a").is_some());
    assert!(json["frames"].get("b").is_some());
    assert!(json["frames"].get("c").is_some());
}

#[test]
fn json_hash_frame_rect_values_match() {
    let atlas = make_atlas(vec![make_frame("s", 10, 20, 64, 48)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let frame = &json["frames"]["s"]["frame"];
    assert_eq!(frame["x"], 10);
    assert_eq!(frame["y"], 20);
    assert_eq!(frame["w"], 64);
    assert_eq!(frame["h"], 48);
}

#[test]
fn json_hash_source_size_values_match() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 64, 48)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let src = &json["frames"]["s"]["sourceSize"];
    assert_eq!(src["w"], 64);
    assert_eq!(src["h"], 48);
}

#[test]
fn json_hash_meta_image_matches_texture_filename() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert_eq!(json["meta"]["image"], "atlas.png");
}

#[test]
fn json_hash_meta_size_matches_atlas_size() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert_eq!(json["meta"]["size"]["w"], 256);
    assert_eq!(json["meta"]["size"]["h"], 128);
}

#[test]
fn json_hash_meta_app_is_fastpack() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert_eq!(json["meta"]["app"], "FastPack");
}

#[test]
fn json_hash_rotated_flag_serialized() {
    let mut frame = make_frame("s", 0, 0, 32, 64);
    frame.rotated = true;
    let atlas = make_atlas(vec![frame]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert_eq!(json["frames"]["s"]["rotated"], true);
}

#[test]
fn json_hash_trimmed_flag_serialized() {
    let mut frame = make_frame("s", 0, 0, 32, 32);
    frame.trimmed = true;
    frame.sprite_source_size = SourceRect {
        x: 4,
        y: 4,
        w: 32,
        h: 32,
    };
    frame.source_size = Size { w: 40, h: 40 };
    let atlas = make_atlas(vec![frame]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert_eq!(json["frames"]["s"]["trimmed"], true);
}

#[test]
fn json_hash_pivot_field_present_when_set() {
    let mut frame = make_frame("s", 0, 0, 32, 32);
    frame.pivot = Some(Point { x: 0.5, y: 0.5 });
    let atlas = make_atlas(vec![frame]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let pivot = &json["frames"]["s"]["pivot"];
    assert!(!pivot.is_null(), "pivot should be present");
    assert!((pivot["x"].as_f64().unwrap() - 0.5).abs() < 1e-6);
    assert!((pivot["y"].as_f64().unwrap() - 0.5).abs() < 1e-6);
}

#[test]
fn json_hash_pivot_field_absent_when_none() {
    let frame = make_frame("s", 0, 0, 32, 32);
    let atlas = make_atlas(vec![frame]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(
        json["frames"]["s"].get("pivot").is_none(),
        "pivot should be absent when None"
    );
}

#[test]
fn json_hash_nine_patch_field_present_when_set() {
    let mut frame = make_frame("s", 0, 0, 32, 32);
    frame.nine_patch = Some(NinePatch {
        top: 4,
        right: 4,
        bottom: 4,
        left: 4,
    });
    let atlas = make_atlas(vec![frame]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let np = &json["frames"]["s"]["ninePatch"];
    assert!(!np.is_null(), "ninePatch should be present");
    assert_eq!(np["top"], 4);
    assert_eq!(np["right"], 4);
    assert_eq!(np["bottom"], 4);
    assert_eq!(np["left"], 4);
}

#[test]
fn json_hash_nine_patch_absent_when_none() {
    let frame = make_frame("s", 0, 0, 32, 32);
    let atlas = make_atlas(vec![frame]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(
        json["frames"]["s"].get("ninePatch").is_none(),
        "ninePatch should be absent when None"
    );
}

#[test]
fn json_hash_alias_of_field_present_when_set() {
    let mut frame = make_frame("copy", 0, 0, 32, 32);
    frame.alias_of = Some("original".to_string());
    let atlas = make_atlas(vec![make_frame("original", 0, 0, 32, 32), frame]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert_eq!(json["frames"]["copy"]["aliasOf"], "original");
}

#[test]
fn json_hash_alias_of_absent_when_none() {
    let frame = make_frame("s", 0, 0, 32, 32);
    let atlas = make_atlas(vec![frame]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(
        json["frames"]["s"].get("aliasOf").is_none(),
        "aliasOf should be absent when None"
    );
}

#[test]
fn json_hash_empty_atlas_produces_valid_json() {
    let atlas = PackedAtlas {
        frames: vec![],
        size: Size { w: 64, h: 64 },
        image: None,
        name: "empty".to_string(),
        scale: 1.0,
    };
    let result = JsonHashExporter.export(&export_input(&atlas));
    assert!(result.is_ok());
    let json: Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(json["frames"].as_object().unwrap().is_empty());
}

#[test]
fn json_hash_format_id_and_extension() {
    assert_eq!(JsonHashExporter.format_id(), "json_hash");
    assert_eq!(JsonHashExporter.file_extension(), "json");
}

// Phaser3Exporter

#[test]
fn phaser3_output_has_textures_and_meta_keys() {
    let atlas = make_atlas(vec![make_frame("hero", 0, 0, 64, 64)]);
    let json: Value =
        serde_json::from_str(&Phaser3Exporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(json.get("textures").is_some(), "missing 'textures' key");
    assert!(json.get("meta").is_some(), "missing 'meta' key");
}

#[test]
fn phaser3_single_sheet_textures_array_has_one_entry() {
    let atlas = make_atlas(vec![make_frame("a", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&Phaser3Exporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert_eq!(json["textures"].as_array().unwrap().len(), 1);
}

#[test]
fn phaser3_texture_entry_has_image_and_frames() {
    let atlas = make_atlas(vec![make_frame("sprite", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&Phaser3Exporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let entry = &json["textures"][0];
    assert_eq!(entry["image"], "atlas.png");
    assert!(entry.get("frames").is_some());
}

#[test]
fn phaser3_frame_uses_filename_key() {
    let atlas = make_atlas(vec![make_frame("player/idle", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&Phaser3Exporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let frames = json["textures"][0]["frames"].as_array().unwrap();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0]["filename"], "player/idle");
}

#[test]
fn phaser3_frame_rect_values_match() {
    let atlas = make_atlas(vec![make_frame("s", 8, 16, 48, 32)]);
    let json: Value =
        serde_json::from_str(&Phaser3Exporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let frame = &json["textures"][0]["frames"][0]["frame"];
    assert_eq!(frame["x"], 8);
    assert_eq!(frame["y"], 16);
    assert_eq!(frame["w"], 48);
    assert_eq!(frame["h"], 32);
}

#[test]
fn phaser3_texture_size_matches_atlas() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&Phaser3Exporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let size = &json["textures"][0]["size"];
    assert_eq!(size["w"], 256);
    assert_eq!(size["h"], 128);
}

#[test]
fn phaser3_meta_version_is_3_0() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&Phaser3Exporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert_eq!(json["meta"]["version"], "3.0");
}

#[test]
fn phaser3_meta_app_is_fastpack() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&Phaser3Exporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert_eq!(json["meta"]["app"], "FastPack");
}

#[test]
fn phaser3_combine_two_sheets_produces_two_textures() {
    let atlas1 = make_atlas(vec![make_frame("a", 0, 0, 32, 32)]);
    let atlas2 = PackedAtlas {
        frames: vec![make_frame("b", 0, 0, 32, 32)],
        size: Size { w: 128, h: 128 },
        image: None,
        name: "atlas".to_string(),
        scale: 1.0,
    };
    let input1 = ExportInput {
        atlas: &atlas1,
        texture_filename: "atlas.png".to_string(),
        pixel_format: "RGBA8888".to_string(),
    };
    let input2 = ExportInput {
        atlas: &atlas2,
        texture_filename: "atlas1.png".to_string(),
        pixel_format: "RGBA8888".to_string(),
    };
    let combined = Phaser3Exporter
        .combine(&[input1, input2])
        .expect("should produce combined output")
        .unwrap();
    let json: Value = serde_json::from_str(&combined).unwrap();
    assert_eq!(json["textures"].as_array().unwrap().len(), 2);
    assert_eq!(json["textures"][0]["image"], "atlas.png");
    assert_eq!(json["textures"][1]["image"], "atlas1.png");
}

#[test]
fn phaser3_combine_returns_some() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let input = export_input(&atlas);
    assert!(
        Phaser3Exporter
            .combine(std::slice::from_ref(&input))
            .is_some(),
        "Phaser3 should support combine()"
    );
}

#[test]
fn phaser3_format_id_and_extension() {
    assert_eq!(Phaser3Exporter.format_id(), "phaser3");
    assert_eq!(Phaser3Exporter.file_extension(), "json");
}

// PixiJsExporter

#[test]
fn pixijs_output_is_json_hash_format() {
    let atlas = make_atlas(vec![make_frame("sprite", 0, 0, 64, 64)]);
    let json: Value =
        serde_json::from_str(&PixiJsExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(json.get("frames").is_some(), "missing 'frames' key");
    assert!(json.get("meta").is_some(), "missing 'meta' key");
}

#[test]
fn pixijs_frame_id_as_hash_key() {
    let atlas = make_atlas(vec![make_frame("player", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&PixiJsExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(json["frames"].get("player").is_some());
}

#[test]
fn pixijs_frame_rect_matches() {
    let atlas = make_atlas(vec![make_frame("s", 10, 20, 30, 40)]);
    let json: Value =
        serde_json::from_str(&PixiJsExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let frame = &json["frames"]["s"]["frame"];
    assert_eq!(frame["x"], 10);
    assert_eq!(frame["y"], 20);
    assert_eq!(frame["w"], 30);
    assert_eq!(frame["h"], 40);
}

#[test]
fn pixijs_format_id_and_extension() {
    assert_eq!(PixiJsExporter.format_id(), "pixijs");
    assert_eq!(PixiJsExporter.file_extension(), "json");
}

#[test]
fn pixijs_combine_returns_none() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let input = export_input(&atlas);
    assert!(
        PixiJsExporter
            .combine(std::slice::from_ref(&input))
            .is_none(),
        "PixiJS should not support combine()"
    );
}
