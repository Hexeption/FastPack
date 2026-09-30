use fastpack_core::types::{
    atlas::{AtlasFrame, PackedAtlas},
    rect::{Point, Rect, Size, SourceRect},
    sprite::NinePatch,
};
use fastpack_formats::{
    exporter::{ExportInput, Exporter},
    formats::{
        cocos2d::Cocos2dExporter, json_array::JsonArrayExporter, json_hash::JsonHashExporter,
        phaser3::Phaser3Exporter, pixijs::PixiJsExporter,
    },
    polygon::build_mesh,
};
use serde_json::Value;

// Helpers

fn make_frame(id: &str, x: u32, y: u32, w: u32, h: u32) -> AtlasFrame {
    AtlasFrame {
        id: id.to_string(),
        frame: Rect { x, y, w, h },
        rotated: false,
        trimmed: false,
        sprite_source_size: SourceRect { x: 0, y: 0, w, h },
        source_size: Size { w, h },
        polygon: None,
        extrude: 0,
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
        hide_name: false,
    }
}

fn hidden_input(atlas: &PackedAtlas) -> ExportInput<'_> {
    ExportInput {
        hide_name: true,
        ..export_input(atlas)
    }
}

fn assert_smartupdate_format(value: &Value) {
    let s = value.as_str().expect("smartupdate should be a string");
    let inner = s
        .strip_prefix("$TexturePacker:SmartUpdate:")
        .and_then(|rest| rest.strip_suffix('$'))
        .unwrap_or_else(|| panic!("unexpected smartupdate wrapper: {s}"));
    let hashes: Vec<&str> = inner.split(':').collect();
    assert_eq!(hashes.len(), 3, "expected three hashes in {s}");
    for h in hashes {
        assert_eq!(h.len(), 32);
        assert!(
            h.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
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
fn json_hash_meta_has_no_smartupdate_by_default() {
    let atlas = make_atlas(vec![make_frame("a", 0, 0, 8, 8)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    assert!(json["meta"].get("smartupdate").is_none());
}

#[test]
fn json_hash_hide_name_uses_texturepacker_meta() {
    let atlas = make_atlas(vec![make_frame("a", 0, 0, 8, 8)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&hidden_input(&atlas)).unwrap()).unwrap();
    assert_eq!(
        json["meta"]["app"],
        "https://www.codeandweb.com/texturepacker"
    );
    assert_eq!(json["meta"]["version"], "3.0");
    assert_smartupdate_format(&json["meta"]["smartupdate"]);
}

#[test]
fn smartupdate_changes_when_frames_change() {
    let a = make_atlas(vec![make_frame("a", 0, 0, 8, 8)]);
    let b = make_atlas(vec![make_frame("a", 8, 0, 8, 8)]);
    let ja: Value =
        serde_json::from_str(&JsonHashExporter.export(&hidden_input(&a)).unwrap()).unwrap();
    let jb: Value =
        serde_json::from_str(&JsonHashExporter.export(&hidden_input(&b)).unwrap()).unwrap();
    assert_ne!(ja["meta"]["smartupdate"], jb["meta"]["smartupdate"]);
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
fn phaser3_hide_name_uses_texturepacker_meta() {
    let atlas = make_atlas(vec![make_frame("a", 0, 0, 8, 8)]);
    let json: Value =
        serde_json::from_str(&Phaser3Exporter.export(&hidden_input(&atlas)).unwrap()).unwrap();
    assert_eq!(
        json["meta"]["app"],
        "https://www.codeandweb.com/texturepacker"
    );
    assert_smartupdate_format(&json["meta"]["smartupdate"]);
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
        hide_name: false,
    };
    let input2 = ExportInput {
        atlas: &atlas2,
        texture_filename: "atlas1.png".to_string(),
        pixel_format: "RGBA8888".to_string(),
        hide_name: false,
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

// Polygon mesh data

/// A 20×20 source trimmed to an 8×6 rectangle at (3,4), with the closed hull
/// ring `compute_convex_hull` produces (first vertex repeated at the end).
fn make_polygon_frame(id: &str, x: u32, y: u32, rotated: bool, extrude: u32) -> AtlasFrame {
    let (w, h) = (8 + 2 * extrude, 6 + 2 * extrude);
    let (w, h) = if rotated { (h, w) } else { (w, h) };
    let pts = [(0.0, 0.0), (8.0, 0.0), (8.0, 6.0), (0.0, 6.0), (0.0, 0.0)];
    AtlasFrame {
        rotated,
        trimmed: true,
        sprite_source_size: SourceRect {
            x: 3,
            y: 4,
            w: 8,
            h: 6,
        },
        source_size: Size { w: 20, h: 20 },
        polygon: Some(pts.iter().map(|&(x, y)| Point { x, y }).collect()),
        extrude,
        ..make_frame(id, x, y, w, h)
    }
}

fn pairs(v: &Value) -> Vec<[i64; 2]> {
    v.as_array()
        .expect("expected array")
        .iter()
        .map(|p| [p[0].as_i64().unwrap(), p[1].as_i64().unwrap()])
        .collect()
}

#[test]
fn json_hash_polygon_frame_has_mesh() {
    let atlas = make_atlas(vec![make_polygon_frame("p", 10, 20, false, 1)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let f = &json["frames"]["p"];
    // Closing vertex dropped; vertices offset by spriteSourceSize.
    assert_eq!(
        pairs(&f["vertices"]),
        vec![[3, 4], [11, 4], [11, 10], [3, 10]]
    );
    // Atlas coords: frame position + extrude border.
    assert_eq!(
        pairs(&f["verticesUV"]),
        vec![[11, 21], [19, 21], [19, 27], [11, 27]]
    );
    assert_eq!(f["triangles"], serde_json::json!([[0, 1, 2], [0, 2, 3]]));
}

#[test]
fn json_hash_polygon_rotated_frame_uv() {
    // Rotated 90° clockwise: the placed rect is 6 wide, 8 tall.
    let atlas = make_atlas(vec![make_polygon_frame("p", 50, 60, true, 0)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let f = &json["frames"]["p"];
    assert_eq!(
        pairs(&f["vertices"]),
        vec![[3, 4], [11, 4], [11, 10], [3, 10]]
    );
    // Source top-left lands at the placed rect's top-right, and so on.
    assert_eq!(
        pairs(&f["verticesUV"]),
        vec![[56, 60], [56, 68], [50, 68], [50, 60]]
    );
    assert_eq!(f["triangles"], serde_json::json!([[0, 1, 2], [0, 2, 3]]));
}

#[test]
fn polygon_rotated_uv_matches_compositor_rotation() {
    // Cross-check against image::imageops::rotate90, which the compositor uses:
    // each source pixel's unit square must map onto the same pixel after rotation.
    let (w, h, e) = (5u32, 3u32, 1u32);
    let (ew, eh) = (w + 2 * e, h + 2 * e);
    let mut src = image::RgbaImage::new(ew, eh);
    for y in 0..eh {
        for x in 0..ew {
            src.put_pixel(x, y, image::Rgba([x as u8, y as u8, 0, 255]));
        }
    }
    let rotated = image::imageops::rotate90(&src);
    for py in 0..h {
        for px in 0..w {
            let (x0, y0) = (px as f32, py as f32);
            let square = vec![
                Point { x: x0, y: y0 },
                Point { x: x0 + 1.0, y: y0 },
                Point {
                    x: x0 + 1.0,
                    y: y0 + 1.0,
                },
                Point { x: x0, y: y0 + 1.0 },
            ];
            let frame = AtlasFrame {
                rotated: true,
                polygon: Some(square),
                extrude: e,
                ..make_frame("p", 0, 0, eh, ew)
            };
            let mesh = build_mesh(&frame).unwrap();
            let ux = mesh.vertices_uv.iter().map(|p| p[0]).min().unwrap() as u32;
            let uy = mesh.vertices_uv.iter().map(|p| p[1]).min().unwrap() as u32;
            let px_rot = rotated.get_pixel(ux, uy);
            assert_eq!(
                (px_rot[0] as u32, px_rot[1] as u32),
                (px + e, py + e),
                "source pixel ({px},{py}) mapped to wrong atlas pixel"
            );
        }
    }
}

#[test]
fn json_hash_non_polygon_frame_omits_mesh() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let json: Value =
        serde_json::from_str(&JsonHashExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let f = &json["frames"]["s"];
    assert!(f.get("vertices").is_none());
    assert!(f.get("verticesUV").is_none());
    assert!(f.get("triangles").is_none());
}

#[test]
fn json_array_polygon_frame_has_mesh() {
    let atlas = make_atlas(vec![
        make_frame("plain", 0, 0, 8, 8),
        make_polygon_frame("p", 10, 20, false, 1),
    ]);
    let json: Value =
        serde_json::from_str(&JsonArrayExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let frames = json["frames"].as_array().unwrap();
    assert!(frames[0].get("vertices").is_none());
    let f = &frames[1];
    assert_eq!(f["filename"], "p");
    assert_eq!(
        pairs(&f["vertices"]),
        vec![[3, 4], [11, 4], [11, 10], [3, 10]]
    );
    assert_eq!(
        pairs(&f["verticesUV"]),
        vec![[11, 21], [19, 21], [19, 27], [11, 27]]
    );
    assert_eq!(f["triangles"], serde_json::json!([[0, 1, 2], [0, 2, 3]]));
}

#[test]
fn pixijs_polygon_frame_has_mesh() {
    let atlas = make_atlas(vec![make_polygon_frame("p", 50, 60, true, 0)]);
    let json: Value =
        serde_json::from_str(&PixiJsExporter.export(&export_input(&atlas)).unwrap()).unwrap();
    let f = &json["frames"]["p"];
    assert_eq!(pairs(&f["vertices"]).len(), 4);
    assert_eq!(
        pairs(&f["verticesUV"]),
        vec![[56, 60], [56, 68], [50, 68], [50, 60]]
    );
    assert_eq!(f["triangles"].as_array().unwrap().len(), 2);
}

#[test]
fn polygon_fan_triangulation_covers_all_vertices() {
    // Hexagon (open ring) -> 4 fan triangles.
    let hex = [
        (2.0, 0.0),
        (6.0, 0.0),
        (8.0, 3.0),
        (6.0, 6.0),
        (2.0, 6.0),
        (0.0, 3.0),
    ];
    let frame = AtlasFrame {
        polygon: Some(hex.iter().map(|&(x, y)| Point { x, y }).collect()),
        ..make_frame("h", 0, 0, 8, 6)
    };
    let mesh = build_mesh(&frame).unwrap();
    assert_eq!(mesh.vertices.len(), 6);
    assert_eq!(
        mesh.triangles,
        vec![[0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 5]]
    );
}
// Cocos2dExporter

/// Return the text from `<key>{key}</key>` up to the next `</dict>`.
fn plist_dict<'a>(plist: &'a str, key: &str) -> &'a str {
    let start = plist
        .find(&format!("<key>{key}</key>"))
        .unwrap_or_else(|| panic!("missing key {key}"));
    let rest = &plist[start..];
    let end = rest.find("</dict>").expect("unterminated dict");
    &rest[..end]
}

/// Return the `<string>` value that follows `<key>{key}</key>` in `body`.
fn plist_string<'a>(body: &'a str, key: &str) -> &'a str {
    let start = body
        .find(&format!("<key>{key}</key>"))
        .unwrap_or_else(|| panic!("missing key {key}"));
    let rest = &body[start..];
    let open = rest.find("<string>").unwrap() + "<string>".len();
    let close = rest.find("</string>").unwrap();
    &rest[open..close]
}

#[test]
fn cocos2d_output_is_plist_with_frames_and_metadata() {
    let atlas = make_atlas(vec![make_frame("hero", 0, 0, 64, 64)]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    assert!(out.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(out.contains("<plist version=\"1.0\">"));
    assert!(out.contains("<key>frames</key>"));
    assert!(out.contains("<key>metadata</key>"));
    assert!(out.trim_end().ends_with("</plist>"));
}

#[test]
fn cocos2d_frame_names_have_png_suffix() {
    let atlas = make_atlas(vec![make_frame("ui/button", 0, 0, 32, 32)]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    assert!(out.contains("<key>ui/button.png</key>"));
}

#[test]
fn cocos2d_frame_rect_strings() {
    let atlas = make_atlas(vec![make_frame("s", 10, 20, 64, 48)]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    let frame = plist_dict(&out, "s.png");
    assert_eq!(plist_string(frame, "textureRect"), "{{10,20},{64,48}}");
    assert_eq!(plist_string(frame, "spriteSize"), "{64,48}");
    assert_eq!(plist_string(frame, "spriteSourceSize"), "{64,48}");
    assert_eq!(plist_string(frame, "spriteOffset"), "{0,0}");
    assert!(frame.contains("<key>textureRotated</key>\n                <false/>"));
}

#[test]
fn cocos2d_trimmed_offset_is_centre_relative_y_up() {
    let mut frame = make_frame("s", 0, 0, 20, 10);
    frame.trimmed = true;
    // 20x10 content at (4, 2) inside a 40x40 source.
    frame.sprite_source_size = SourceRect {
        x: 4,
        y: 2,
        w: 20,
        h: 10,
    };
    frame.source_size = Size { w: 40, h: 40 };
    let atlas = make_atlas(vec![frame]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    let frame = plist_dict(&out, "s.png");
    // x: 4 + 10 - 20 = -6; y: 20 - (2 + 5) = 13.
    assert_eq!(plist_string(frame, "spriteOffset"), "{-6,13}");
    assert_eq!(plist_string(frame, "spriteSourceSize"), "{40,40}");
    assert_eq!(plist_string(frame, "spriteSize"), "{20,10}");
}

#[test]
fn cocos2d_odd_offset_keeps_half_pixels() {
    let mut frame = make_frame("s", 0, 0, 4, 4);
    frame.trimmed = true;
    frame.source_size = Size { w: 5, h: 5 };
    let atlas = make_atlas(vec![frame]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    assert_eq!(
        plist_string(plist_dict(&out, "s.png"), "spriteOffset"),
        "{-0.5,0.5}"
    );
}

#[test]
fn cocos2d_rotated_frame_uses_upright_size() {
    // Packed footprint is 64 wide x 32 tall; the sprite itself is 32x64.
    let mut frame = make_frame("s", 2, 4, 64, 32);
    frame.rotated = true;
    frame.sprite_source_size = SourceRect {
        x: 0,
        y: 0,
        w: 32,
        h: 64,
    };
    frame.source_size = Size { w: 32, h: 64 };
    let atlas = make_atlas(vec![frame]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    let frame = plist_dict(&out, "s.png");
    assert_eq!(plist_string(frame, "textureRect"), "{{2,4},{32,64}}");
    assert_eq!(plist_string(frame, "spriteSize"), "{32,64}");
    assert!(frame.contains("<key>textureRotated</key>\n                <true/>"));
}

#[test]
fn cocos2d_aliases_fold_into_canonical_frame() {
    let mut copy = make_frame("copy", 0, 0, 32, 32);
    copy.alias_of = Some("original".to_string());
    let atlas = make_atlas(vec![make_frame("original", 0, 0, 32, 32), copy]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    assert!(!out.contains("<key>copy.png</key>"));
    let frame = plist_dict(&out, "original.png");
    assert!(frame.contains("<string>copy.png</string>"));
}

#[test]
fn cocos2d_alias_without_canonical_on_sheet_is_standalone() {
    let mut copy = make_frame("copy", 0, 0, 32, 32);
    copy.alias_of = Some("elsewhere".to_string());
    let atlas = make_atlas(vec![copy]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    assert!(out.contains("<key>copy.png</key>"));
}

#[test]
fn cocos2d_pivot_written_as_y_up_anchor() {
    let mut frame = make_frame("s", 0, 0, 32, 32);
    frame.pivot = Some(Point { x: 0.25, y: 0.0 });
    let atlas = make_atlas(vec![frame]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    assert_eq!(
        plist_string(plist_dict(&out, "s.png"), "anchor"),
        "{0.25,1}"
    );
}

#[test]
fn cocos2d_metadata_values() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    let meta = plist_dict(&out, "metadata");
    assert!(meta.contains("<key>format</key>\n            <integer>3</integer>"));
    assert_eq!(plist_string(meta, "textureFileName"), "atlas.png");
    assert_eq!(plist_string(meta, "realTextureFileName"), "atlas.png");
    assert_eq!(plist_string(meta, "size"), "{256,128}");
    assert_eq!(plist_string(meta, "pixelFormat"), "RGBA8888");
    assert!(!meta.contains("smartupdate"));
}

#[test]
fn cocos2d_hide_name_adds_smartupdate() {
    let atlas = make_atlas(vec![make_frame("s", 0, 0, 32, 32)]);
    let out = Cocos2dExporter.export(&hidden_input(&atlas)).unwrap();
    let meta = plist_dict(&out, "metadata");
    assert_smartupdate_format(&Value::String(
        plist_string(meta, "smartupdate").to_string(),
    ));
}

#[test]
fn cocos2d_escapes_xml_in_names() {
    let atlas = make_atlas(vec![make_frame("a&b<c>", 0, 0, 8, 8)]);
    let out = Cocos2dExporter.export(&export_input(&atlas)).unwrap();
    assert!(out.contains("<key>a&amp;b&lt;c&gt;.png</key>"));
}

#[test]
fn cocos2d_format_id_and_extension() {
    assert_eq!(Cocos2dExporter.format_id(), "cocos2d");
    assert_eq!(Cocos2dExporter.file_extension(), "plist");
}
