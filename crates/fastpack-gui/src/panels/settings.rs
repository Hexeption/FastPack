use eframe::egui;
use fastpack_core::types::{
    config::{
        AlgorithmConfig, LayoutConfig, MaxRectsHeuristic, PackMode, ScaleMode, ScaleVariant,
        SizeConstraint, SpriteConfig, TrimMode,
    },
    pixel_format::{PixelFormat, TextureFormat},
};

use crate::state::{AppState, SettingsTab};

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        for (tab, label) in [
            (SettingsTab::Layout, "Layout"),
            (SettingsTab::Sprites, "Sprites"),
            (SettingsTab::Output, "Output"),
            (SettingsTab::Variants, "Variants"),
        ] {
            if ui
                .selectable_label(state.settings_tab == tab, label)
                .clicked()
            {
                state.settings_tab = tab;
            }
        }
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| match state.settings_tab {
        SettingsTab::Layout => show_layout(ui, state),
        SettingsTab::Sprites => show_sprites(ui, state),
        SettingsTab::Output => show_output(ui, state),
        SettingsTab::Variants => show_variants(ui, state),
    });
}

fn show_layout(ui: &mut egui::Ui, state: &mut AppState) {
    let cfg = &mut state.project.config.layout;
    let dirty = &mut state.dirty;

    ui.strong("Atlas Size");
    egui::Grid::new("layout_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Max width");
            if ui
                .add(egui::DragValue::new(&mut cfg.max_width).range(1..=16384))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Max height");
            if ui
                .add(egui::DragValue::new(&mut cfg.max_height).range(1..=16384))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Fixed width");
            let mut has_fixed_w = cfg.fixed_width.is_some();
            ui.horizontal(|ui| {
                if ui.checkbox(&mut has_fixed_w, "").changed() {
                    cfg.fixed_width = if has_fixed_w { Some(256) } else { None };
                    *dirty = true;
                }
                if let Some(ref mut v) = cfg.fixed_width {
                    if ui.add(egui::DragValue::new(v).range(1..=16384)).changed() {
                        *dirty = true;
                    }
                }
            });
            ui.end_row();

            ui.label("Fixed height");
            let mut has_fixed_h = cfg.fixed_height.is_some();
            ui.horizontal(|ui| {
                if ui.checkbox(&mut has_fixed_h, "").changed() {
                    cfg.fixed_height = if has_fixed_h { Some(256) } else { None };
                    *dirty = true;
                }
                if let Some(ref mut v) = cfg.fixed_height {
                    if ui.add(egui::DragValue::new(v).range(1..=16384)).changed() {
                        *dirty = true;
                    }
                }
            });
            ui.end_row();

            ui.label("Size constraint");
            let prev = cfg.size_constraint;
            egui::ComboBox::from_id_salt("size_constraint")
                .selected_text(match cfg.size_constraint {
                    SizeConstraint::AnySize => "Any",
                    SizeConstraint::Pot => "Power of 2",
                    SizeConstraint::MultipleOf4 => "Multiple of 4",
                    SizeConstraint::WordAligned => "Word aligned",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cfg.size_constraint, SizeConstraint::AnySize, "Any");
                    ui.selectable_value(&mut cfg.size_constraint, SizeConstraint::Pot, "Power of 2");
                    ui.selectable_value(
                        &mut cfg.size_constraint,
                        SizeConstraint::MultipleOf4,
                        "Multiple of 4",
                    );
                    ui.selectable_value(
                        &mut cfg.size_constraint,
                        SizeConstraint::WordAligned,
                        "Word aligned",
                    );
                });
            if cfg.size_constraint != prev {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Force square");
            if ui.checkbox(&mut cfg.force_square, "").changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Allow rotation");
            if ui.checkbox(&mut cfg.allow_rotation, "").changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Border padding");
            if ui
                .add(egui::DragValue::new(&mut cfg.border_padding).range(0..=64))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Shape padding");
            if ui
                .add(egui::DragValue::new(&mut cfg.shape_padding).range(0..=64))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.strong("Algorithm");

    let algo = &mut state.project.config.algorithm;
    let algo_label = match algo {
        AlgorithmConfig::Grid { .. } => "Grid",
        AlgorithmConfig::Basic => "Basic",
        AlgorithmConfig::MaxRects { .. } => "MaxRects",
        AlgorithmConfig::Polygon => "Polygon",
    };
    let old_label = algo_label;
    let mut new_algo: Option<AlgorithmConfig> = None;
    egui::ComboBox::from_id_salt("settings_algo")
        .selected_text(algo_label)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(
                    matches!(algo, AlgorithmConfig::Grid { .. }),
                    "Grid",
                )
                .clicked()
            {
                new_algo = Some(AlgorithmConfig::Grid {
                    cell_width: 0,
                    cell_height: 0,
                });
            }
            if ui
                .selectable_label(matches!(algo, AlgorithmConfig::Basic), "Basic")
                .clicked()
            {
                new_algo = Some(AlgorithmConfig::Basic);
            }
            if ui
                .selectable_label(matches!(algo, AlgorithmConfig::MaxRects { .. }), "MaxRects")
                .clicked()
            {
                new_algo = Some(AlgorithmConfig::MaxRects {
                    heuristic: MaxRectsHeuristic::BestShortSideFit,
                });
            }
            if ui
                .selectable_label(matches!(algo, AlgorithmConfig::Polygon), "Polygon")
                .clicked()
            {
                new_algo = Some(AlgorithmConfig::Polygon);
            }
        });
    if let Some(a) = new_algo {
        state.project.config.algorithm = a;
        state.dirty = true;
    }

    if let AlgorithmConfig::MaxRects { heuristic } = &mut state.project.config.algorithm {
        ui.horizontal(|ui| {
            ui.label("Heuristic");
            let prev = *heuristic;
            egui::ComboBox::from_id_salt("maxrects_heuristic")
                .selected_text(match heuristic {
                    MaxRectsHeuristic::BestShortSideFit => "Best short side",
                    MaxRectsHeuristic::BestLongSideFit => "Best long side",
                    MaxRectsHeuristic::BestAreaFit => "Best area",
                    MaxRectsHeuristic::BottomLeftRule => "Bottom-left",
                    MaxRectsHeuristic::ContactPointRule => "Contact point",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        heuristic,
                        MaxRectsHeuristic::BestShortSideFit,
                        "Best short side",
                    );
                    ui.selectable_value(
                        heuristic,
                        MaxRectsHeuristic::BestLongSideFit,
                        "Best long side",
                    );
                    ui.selectable_value(heuristic, MaxRectsHeuristic::BestAreaFit, "Best area");
                    ui.selectable_value(
                        heuristic,
                        MaxRectsHeuristic::BottomLeftRule,
                        "Bottom-left",
                    );
                    ui.selectable_value(
                        heuristic,
                        MaxRectsHeuristic::ContactPointRule,
                        "Contact point",
                    );
                });
            if *heuristic != prev {
                state.dirty = true;
            }
        });
    }

    if let AlgorithmConfig::Grid {
        cell_width,
        cell_height,
    } = &mut state.project.config.algorithm
    {
        egui::Grid::new("grid_algo")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label("Cell width (0=auto)");
                if ui.add(egui::DragValue::new(cell_width)).changed() {
                    state.dirty = true;
                }
                ui.end_row();
                ui.label("Cell height (0=auto)");
                if ui.add(egui::DragValue::new(cell_height)).changed() {
                    state.dirty = true;
                }
                ui.end_row();
            });
    }
}

fn show_sprites(ui: &mut egui::Ui, state: &mut AppState) {
    let cfg = &mut state.project.config.sprites;
    let dirty = &mut state.dirty;

    egui::Grid::new("sprites_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Trim mode");
            let prev = cfg.trim_mode;
            egui::ComboBox::from_id_salt("trim_mode")
                .selected_text(match cfg.trim_mode {
                    TrimMode::None => "None",
                    TrimMode::Trim => "Trim",
                    TrimMode::Crop => "Crop",
                    TrimMode::CropKeepPos => "Crop keep pos",
                    TrimMode::Polygon => "Polygon",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cfg.trim_mode, TrimMode::None, "None");
                    ui.selectable_value(&mut cfg.trim_mode, TrimMode::Trim, "Trim");
                    ui.selectable_value(&mut cfg.trim_mode, TrimMode::Crop, "Crop");
                    ui.selectable_value(&mut cfg.trim_mode, TrimMode::CropKeepPos, "Crop keep pos");
                    ui.selectable_value(&mut cfg.trim_mode, TrimMode::Polygon, "Polygon");
                });
            if cfg.trim_mode != prev {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Trim threshold");
            if ui
                .add(egui::Slider::new(&mut cfg.trim_threshold, 0..=255))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Trim margin");
            if ui
                .add(egui::DragValue::new(&mut cfg.trim_margin).range(0..=32))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Extrude");
            if ui
                .add(egui::DragValue::new(&mut cfg.extrude).range(0..=16))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Divisor X");
            if ui
                .add(egui::DragValue::new(&mut cfg.common_divisor_x).range(0..=64))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Divisor Y");
            if ui
                .add(egui::DragValue::new(&mut cfg.common_divisor_y).range(0..=64))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Detect aliases");
            if ui.checkbox(&mut cfg.detect_aliases, "").changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Pivot X");
            if ui
                .add(egui::Slider::new(&mut cfg.default_pivot.x, 0.0..=1.0))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Pivot Y");
            if ui
                .add(egui::Slider::new(&mut cfg.default_pivot.y, 0.0..=1.0))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();
        });
}

fn show_output(ui: &mut egui::Ui, state: &mut AppState) {
    let cfg = &mut state.project.config.output;
    let dirty = &mut state.dirty;

    egui::Grid::new("output_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Name");
            if ui.text_edit_singleline(&mut cfg.name).changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Directory");
            let mut dir_str = cfg.directory.to_string_lossy().into_owned();
            if ui.text_edit_singleline(&mut dir_str).changed() {
                cfg.directory = std::path::PathBuf::from(&dir_str);
                *dirty = true;
            }
            ui.end_row();

            ui.label("Texture format");
            let prev_tf = cfg.texture_format;
            egui::ComboBox::from_id_salt("texture_format")
                .selected_text(match cfg.texture_format {
                    TextureFormat::Png => "PNG",
                    TextureFormat::Jpeg => "JPEG",
                    TextureFormat::WebP => "WebP",
                    _ => "Other",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cfg.texture_format, TextureFormat::Png, "PNG");
                    ui.selectable_value(&mut cfg.texture_format, TextureFormat::Jpeg, "JPEG");
                    ui.selectable_value(&mut cfg.texture_format, TextureFormat::WebP, "WebP");
                });
            if cfg.texture_format != prev_tf {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Pixel format");
            let prev_pf = cfg.pixel_format;
            egui::ComboBox::from_id_salt("pixel_format")
                .selected_text(match cfg.pixel_format {
                    PixelFormat::Rgba8888 => "RGBA8888",
                    PixelFormat::Rgb888 => "RGB888",
                    PixelFormat::Rgb565 => "RGB565",
                    PixelFormat::Rgba4444 => "RGBA4444",
                    _ => "Other",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cfg.pixel_format, PixelFormat::Rgba8888, "RGBA8888");
                    ui.selectable_value(&mut cfg.pixel_format, PixelFormat::Rgb888, "RGB888");
                    ui.selectable_value(&mut cfg.pixel_format, PixelFormat::Rgb565, "RGB565");
                    ui.selectable_value(&mut cfg.pixel_format, PixelFormat::Rgba4444, "RGBA4444");
                });
            if cfg.pixel_format != prev_pf {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Premultiply alpha");
            if ui.checkbox(&mut cfg.premultiply_alpha, "").changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Data format");
            let prev_df = cfg.data_format.clone();
            egui::ComboBox::from_id_salt("data_format")
                .selected_text(&cfg.data_format)
                .show_ui(ui, |ui| {
                    for fmt in ["json_hash", "phaser3", "pixijs"] {
                        ui.selectable_value(&mut cfg.data_format, fmt.to_string(), fmt);
                    }
                });
            if cfg.data_format != prev_df {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Quality");
            if ui
                .add(egui::Slider::new(&mut cfg.quality, 0..=100))
                .changed()
            {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Texture prefix");
            if ui.text_edit_singleline(&mut cfg.texture_path_prefix).changed() {
                *dirty = true;
            }
            ui.end_row();
        });
}

fn show_variants(ui: &mut egui::Ui, state: &mut AppState) {
    let mut remove_idx: Option<usize> = None;

    for (i, variant) in state.project.config.variants.iter_mut().enumerate() {
        ui.group(|ui| {
            egui::Grid::new(format!("variant_{i}"))
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Scale");
                    if ui
                        .add(
                            egui::DragValue::new(&mut variant.scale)
                                .range(0.01..=8.0)
                                .speed(0.01),
                        )
                        .changed()
                    {
                        state.dirty = true;
                    }
                    ui.end_row();

                    ui.label("Suffix");
                    if ui.text_edit_singleline(&mut variant.suffix).changed() {
                        state.dirty = true;
                    }
                    ui.end_row();

                    ui.label("Scale mode");
                    let prev = variant.scale_mode;
                    egui::ComboBox::from_id_salt(format!("scale_mode_{i}"))
                        .selected_text(match variant.scale_mode {
                            ScaleMode::Smooth => "Smooth",
                            ScaleMode::Fast => "Fast",
                            ScaleMode::Scale2x => "Scale2x",
                            ScaleMode::Scale3x => "Scale3x",
                            ScaleMode::Hq2x => "HQ2x",
                            _ => "Other",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut variant.scale_mode, ScaleMode::Smooth, "Smooth");
                            ui.selectable_value(&mut variant.scale_mode, ScaleMode::Fast, "Fast");
                            ui.selectable_value(
                                &mut variant.scale_mode,
                                ScaleMode::Scale2x,
                                "Scale2x",
                            );
                            ui.selectable_value(
                                &mut variant.scale_mode,
                                ScaleMode::Scale3x,
                                "Scale3x",
                            );
                            ui.selectable_value(&mut variant.scale_mode, ScaleMode::Hq2x, "HQ2x");
                        });
                    if variant.scale_mode != prev {
                        state.dirty = true;
                    }
                    ui.end_row();
                });
            if ui.small_button("Remove").clicked() {
                remove_idx = Some(i);
            }
        });
        ui.add_space(4.0);
    }

    if let Some(i) = remove_idx {
        state.project.config.variants.remove(i);
        state.dirty = true;
    }

    if ui.button("Add Variant").clicked() {
        state.project.config.variants.push(ScaleVariant::default());
        state.dirty = true;
    }
}
