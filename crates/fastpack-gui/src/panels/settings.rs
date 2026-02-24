use std::path::PathBuf;

use eframe::egui;
use fastpack_core::types::{
    config::{
        AlgorithmConfig, MaxRectsHeuristic, PackMode, ScaleMode, ScaleVariant, SizeConstraint,
        TrimMode,
    },
    pixel_format::{PixelFormat, TextureFormat},
};

use crate::state::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(2.0);

        section(ui, "Texture", true, |ui| {
            show_texture(ui, state);
        });
        section(ui, "Layout", true, |ui| {
            show_layout(ui, state);
        });
        section(ui, "Sprites", true, |ui| {
            show_sprites(ui, state);
        });
        section(ui, "Variants", false, |ui| {
            show_variants(ui, state);
        });
    });
}

fn section(ui: &mut egui::Ui, label: &str, open: bool, body: impl FnOnce(&mut egui::Ui)) {
    let id = egui::Id::new(("settings_section", label));
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, open)
        .show_header(ui, |ui| {
            ui.strong(label);
        })
        .body(|ui| {
            ui.add_space(2.0);
            body(ui);
            ui.add_space(4.0);
        });
    ui.separator();
}

fn setting_row(ui: &mut egui::Ui, label: &str, widget: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [130.0, 20.0],
            egui::Label::new(egui::RichText::new(label).small()),
        );
        widget(ui);
    });
}

fn show_texture(ui: &mut egui::Ui, state: &mut AppState) {
    let cfg = &mut state.project.config.output;
    let dirty = &mut state.dirty;

    setting_row(ui, "Name", |ui| {
        if ui
            .add(egui::TextEdit::singleline(&mut cfg.name).desired_width(f32::INFINITY))
            .changed()
        {
            *dirty = true;
        }
    });

    setting_row(ui, "Directory", |ui| {
        let mut dir_str = cfg.directory.to_string_lossy().into_owned();
        if ui
            .add(egui::TextEdit::singleline(&mut dir_str).desired_width(120.0))
            .changed()
        {
            cfg.directory = PathBuf::from(&dir_str);
            *dirty = true;
        }
        if ui.button("Browse...").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                cfg.directory = path;
                *dirty = true;
            }
        }
    });

    setting_row(ui, "Texture format", |ui| {
        let prev = cfg.texture_format;
        egui::ComboBox::from_id_salt("texture_format")
            .selected_text(match cfg.texture_format {
                TextureFormat::Png => "PNG",
                TextureFormat::Jpeg => "JPEG",
                TextureFormat::WebP => "WebP",
                _ => "Other",
            })
            .width(120.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut cfg.texture_format, TextureFormat::Png, "PNG");
                ui.selectable_value(&mut cfg.texture_format, TextureFormat::Jpeg, "JPEG");
                ui.selectable_value(&mut cfg.texture_format, TextureFormat::WebP, "WebP");
            });
        if cfg.texture_format != prev {
            *dirty = true;
        }
    });

    setting_row(ui, "Pixel format", |ui| {
        let prev = cfg.pixel_format;
        egui::ComboBox::from_id_salt("pixel_format")
            .selected_text(match cfg.pixel_format {
                PixelFormat::Rgba8888 => "RGBA8888",
                PixelFormat::Rgb888 => "RGB888",
                PixelFormat::Rgb565 => "RGB565",
                PixelFormat::Rgba4444 => "RGBA4444",
                _ => "Other",
            })
            .width(120.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut cfg.pixel_format, PixelFormat::Rgba8888, "RGBA8888");
                ui.selectable_value(&mut cfg.pixel_format, PixelFormat::Rgb888, "RGB888");
                ui.selectable_value(&mut cfg.pixel_format, PixelFormat::Rgb565, "RGB565");
                ui.selectable_value(&mut cfg.pixel_format, PixelFormat::Rgba4444, "RGBA4444");
            });
        if cfg.pixel_format != prev {
            *dirty = true;
        }
    });

    setting_row(ui, "Quality", |ui| {
        if ui
            .add(egui::Slider::new(&mut cfg.quality, 0..=100).suffix("%"))
            .changed()
        {
            *dirty = true;
        }
    });

    setting_row(ui, "Data format", |ui| {
        let prev = cfg.data_format.clone();
        egui::ComboBox::from_id_salt("data_format")
            .selected_text(&cfg.data_format)
            .width(120.0)
            .show_ui(ui, |ui| {
                for fmt in ["json_hash", "phaser3", "pixijs"] {
                    ui.selectable_value(&mut cfg.data_format, fmt.to_string(), fmt);
                }
            });
        if cfg.data_format != prev {
            *dirty = true;
        }
    });

    setting_row(ui, "Premultiply alpha", |ui| {
        if ui.checkbox(&mut cfg.premultiply_alpha, "").changed() {
            *dirty = true;
        }
    });

    setting_row(ui, "Texture prefix", |ui| {
        if ui
            .add(
                egui::TextEdit::singleline(&mut cfg.texture_path_prefix)
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            *dirty = true;
        }
    });
}

fn show_layout(ui: &mut egui::Ui, state: &mut AppState) {
    let AppState {
        project,
        dirty,
        pending,
        ..
    } = state;
    let cfg = &mut project.config.layout;

    setting_row(ui, "Max size", |ui| {
        if ui
            .add(
                egui::DragValue::new(&mut cfg.max_width)
                    .range(1..=16384)
                    .prefix("W "),
            )
            .changed()
        {
            *dirty = true;
            pending.pack = true;
        }
        ui.label("x");
        if ui
            .add(
                egui::DragValue::new(&mut cfg.max_height)
                    .range(1..=16384)
                    .prefix("H "),
            )
            .changed()
        {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Fixed width", |ui| {
        let mut enabled = cfg.fixed_width.is_some();
        if ui.checkbox(&mut enabled, "").changed() {
            cfg.fixed_width = if enabled { Some(256) } else { None };
            *dirty = true;
            pending.pack = true;
        }
        if let Some(ref mut v) = cfg.fixed_width {
            if ui.add(egui::DragValue::new(v).range(1..=16384)).changed() {
                *dirty = true;
                pending.pack = true;
            }
        }
    });

    setting_row(ui, "Fixed height", |ui| {
        let mut enabled = cfg.fixed_height.is_some();
        if ui.checkbox(&mut enabled, "").changed() {
            cfg.fixed_height = if enabled { Some(256) } else { None };
            *dirty = true;
            pending.pack = true;
        }
        if let Some(ref mut v) = cfg.fixed_height {
            if ui.add(egui::DragValue::new(v).range(1..=16384)).changed() {
                *dirty = true;
                pending.pack = true;
            }
        }
    });

    setting_row(ui, "Size constraints", |ui| {
        let prev = cfg.size_constraint;
        egui::ComboBox::from_id_salt("size_constraint")
            .selected_text(match cfg.size_constraint {
                SizeConstraint::AnySize => "Any size",
                SizeConstraint::Pot => "Power of 2",
                SizeConstraint::MultipleOf4 => "Multiple of 4",
                SizeConstraint::WordAligned => "Word aligned",
            })
            .width(120.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut cfg.size_constraint,
                    SizeConstraint::AnySize,
                    "Any size",
                );
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
            pending.pack = true;
        }
    });

    setting_row(ui, "Force squared", |ui| {
        if ui.checkbox(&mut cfg.force_square, "").changed() {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Allow rotation", |ui| {
        if ui.checkbox(&mut cfg.allow_rotation, "").changed() {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Border padding", |ui| {
        if ui
            .add(egui::DragValue::new(&mut cfg.border_padding).range(0..=64))
            .changed()
        {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Shape padding", |ui| {
        if ui
            .add(egui::DragValue::new(&mut cfg.shape_padding).range(0..=64))
            .changed()
        {
            *dirty = true;
            pending.pack = true;
        }
    });

    ui.add_space(4.0);

    // Algorithm
    let algo = &mut project.config.algorithm;
    let algo_label = match algo {
        AlgorithmConfig::Grid { .. } => "Grid",
        AlgorithmConfig::Basic => "Basic",
        AlgorithmConfig::MaxRects { .. } => "MaxRects",
        AlgorithmConfig::Polygon => "Polygon",
    };
    let mut new_algo: Option<AlgorithmConfig> = None;
    setting_row(ui, "Algorithm", |ui| {
        egui::ComboBox::from_id_salt("settings_algo")
            .selected_text(algo_label)
            .width(120.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(matches!(algo, AlgorithmConfig::Grid { .. }), "Grid")
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
    });
    if let Some(a) = new_algo {
        project.config.algorithm = a;
        *dirty = true;
        pending.pack = true;
    }

    if let AlgorithmConfig::MaxRects { heuristic } = &mut project.config.algorithm {
        setting_row(ui, "Heuristics", |ui| {
            let prev = *heuristic;
            egui::ComboBox::from_id_salt("maxrects_heuristic")
                .selected_text(match heuristic {
                    MaxRectsHeuristic::BestShortSideFit => "ShortSideFit",
                    MaxRectsHeuristic::BestLongSideFit => "LongSideFit",
                    MaxRectsHeuristic::BestAreaFit => "AreaFit",
                    MaxRectsHeuristic::BottomLeftRule => "BottomLeft",
                    MaxRectsHeuristic::ContactPointRule => "ContactPoint",
                })
                .width(120.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        heuristic,
                        MaxRectsHeuristic::BestShortSideFit,
                        "ShortSideFit",
                    );
                    ui.selectable_value(
                        heuristic,
                        MaxRectsHeuristic::BestLongSideFit,
                        "LongSideFit",
                    );
                    ui.selectable_value(heuristic, MaxRectsHeuristic::BestAreaFit, "AreaFit");
                    ui.selectable_value(heuristic, MaxRectsHeuristic::BottomLeftRule, "BottomLeft");
                    ui.selectable_value(
                        heuristic,
                        MaxRectsHeuristic::ContactPointRule,
                        "ContactPoint",
                    );
                });
            if *heuristic != prev {
                *dirty = true;
                pending.pack = true;
            }
        });
    }

    if let AlgorithmConfig::Grid {
        cell_width,
        cell_height,
    } = &mut project.config.algorithm
    {
        setting_row(ui, "Cell width (0=auto)", |ui| {
            if ui.add(egui::DragValue::new(cell_width)).changed() {
                *dirty = true;
                pending.pack = true;
            }
        });
        setting_row(ui, "Cell height (0=auto)", |ui| {
            if ui.add(egui::DragValue::new(cell_height)).changed() {
                *dirty = true;
                pending.pack = true;
            }
        });
    }

    // Pack mode
    setting_row(ui, "Pack", |ui| {
        let prev = cfg.pack_mode;
        egui::ComboBox::from_id_salt("pack_mode")
            .selected_text(match cfg.pack_mode {
                PackMode::Fast => "Fast",
                PackMode::Good => "Good",
                PackMode::Best => "Best",
            })
            .width(120.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut cfg.pack_mode, PackMode::Fast, "Fast");
                ui.selectable_value(&mut cfg.pack_mode, PackMode::Good, "Good");
                ui.selectable_value(&mut cfg.pack_mode, PackMode::Best, "Best");
            });
        if cfg.pack_mode != prev {
            *dirty = true;
            pending.pack = true;
        }
    });
}

fn show_sprites(ui: &mut egui::Ui, state: &mut AppState) {
    let AppState {
        project,
        dirty,
        pending,
        ..
    } = state;
    let cfg = &mut project.config.sprites;

    setting_row(ui, "Trim mode", |ui| {
        let prev = cfg.trim_mode;
        egui::ComboBox::from_id_salt("trim_mode")
            .selected_text(match cfg.trim_mode {
                TrimMode::None => "None",
                TrimMode::Trim => "Trim",
                TrimMode::Crop => "Crop",
                TrimMode::CropKeepPos => "Crop keep pos",
                TrimMode::Polygon => "Polygon",
            })
            .width(120.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut cfg.trim_mode, TrimMode::None, "None");
                ui.selectable_value(&mut cfg.trim_mode, TrimMode::Trim, "Trim");
                ui.selectable_value(&mut cfg.trim_mode, TrimMode::Crop, "Crop");
                ui.selectable_value(&mut cfg.trim_mode, TrimMode::CropKeepPos, "Crop keep pos");
                ui.selectable_value(&mut cfg.trim_mode, TrimMode::Polygon, "Polygon");
            });
        if cfg.trim_mode != prev {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Trim Margin", |ui| {
        if ui
            .add(egui::DragValue::new(&mut cfg.trim_margin).range(0..=32))
            .changed()
        {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Transparency Threshold", |ui| {
        if ui
            .add(egui::DragValue::new(&mut cfg.trim_threshold).range(0..=255))
            .changed()
        {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Extrude", |ui| {
        if ui
            .add(egui::DragValue::new(&mut cfg.extrude).range(0..=16))
            .changed()
        {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Common divisor x", |ui| {
        if ui
            .add(egui::DragValue::new(&mut cfg.common_divisor_x).range(1..=64))
            .changed()
        {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Common divisor y", |ui| {
        if ui
            .add(egui::DragValue::new(&mut cfg.common_divisor_y).range(1..=64))
            .changed()
        {
            *dirty = true;
            pending.pack = true;
        }
    });

    setting_row(ui, "Pivot X", |ui| {
        if ui
            .add(egui::Slider::new(&mut cfg.default_pivot.x, 0.0..=1.0))
            .changed()
        {
            *dirty = true;
        }
    });

    setting_row(ui, "Pivot Y", |ui| {
        if ui
            .add(egui::Slider::new(&mut cfg.default_pivot.y, 0.0..=1.0))
            .changed()
        {
            *dirty = true;
        }
    });

    setting_row(ui, "Detect identical sprites", |ui| {
        if ui.checkbox(&mut cfg.detect_aliases, "").changed() {
            *dirty = true;
            pending.pack = true;
        }
    });
}

fn show_variants(ui: &mut egui::Ui, state: &mut AppState) {
    let mut remove_idx: Option<usize> = None;

    for (i, variant) in state.project.config.variants.iter_mut().enumerate() {
        ui.group(|ui| {
            setting_row(ui, "Scale", |ui| {
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
            });
            setting_row(ui, "Suffix", |ui| {
                if ui.text_edit_singleline(&mut variant.suffix).changed() {
                    state.dirty = true;
                }
            });
            setting_row(ui, "Scale mode", |ui| {
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
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut variant.scale_mode, ScaleMode::Smooth, "Smooth");
                        ui.selectable_value(&mut variant.scale_mode, ScaleMode::Fast, "Fast");
                        ui.selectable_value(&mut variant.scale_mode, ScaleMode::Scale2x, "Scale2x");
                        ui.selectable_value(&mut variant.scale_mode, ScaleMode::Scale3x, "Scale3x");
                        ui.selectable_value(&mut variant.scale_mode, ScaleMode::Hq2x, "HQ2x");
                    });
                if variant.scale_mode != prev {
                    state.dirty = true;
                }
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
