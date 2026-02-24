use eframe::egui;
use fastpack_core::types::config::SpriteOverride;
use rust_i18n::t;

use crate::state::{AppState, FrameInfo, SheetData};

const THUMB_SIZE: f32 = 20.0;

enum TreeNode {
    Folder {
        name: String,
        full_path: String,
        children: Vec<TreeNode>,
    },
    Sprite {
        name: String,
        frame_idx: usize,
    },
}

fn insert_frame(nodes: &mut Vec<TreeNode>, path_prefix: &str, parts: &[&str], frame_idx: usize) {
    if parts.is_empty() {
        return;
    }
    if parts.len() == 1 {
        nodes.push(TreeNode::Sprite {
            name: parts[0].to_string(),
            frame_idx,
        });
        return;
    }
    let folder_name = parts[0];
    let full_path = if path_prefix.is_empty() {
        folder_name.to_string()
    } else {
        format!("{path_prefix}/{folder_name}")
    };
    let pos = nodes
        .iter()
        .position(|n| matches!(n, TreeNode::Folder { name, .. } if name == folder_name));
    if let Some(i) = pos {
        if let TreeNode::Folder { children, .. } = &mut nodes[i] {
            insert_frame(children, &full_path, &parts[1..], frame_idx);
        }
    } else {
        let mut children = Vec::new();
        insert_frame(&mut children, &full_path, &parts[1..], frame_idx);
        nodes.push(TreeNode::Folder {
            name: folder_name.to_string(),
            full_path,
            children,
        });
    }
}

fn build_tree(frames: &[FrameInfo]) -> Vec<TreeNode> {
    let mut roots: Vec<TreeNode> = Vec::new();
    for (i, frame) in frames.iter().enumerate() {
        let parts: Vec<&str> = frame.id.split('/').collect();
        insert_frame(&mut roots, "", &parts, i);
    }
    roots
}

fn draw_thumbnail(
    ui: &mut egui::Ui,
    frame: &FrameInfo,
    sheets: &[SheetData],
    atlas_textures: &[egui::TextureHandle],
) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(THUMB_SIZE, THUMB_SIZE), egui::Sense::hover());

    let si = frame.sheet_idx;
    if si >= atlas_textures.len() || si >= sheets.len() {
        return;
    }
    let sheet = &sheets[si];
    let aw = sheet.width as f32;
    let ah = sheet.height as f32;
    if aw <= 0.0 || ah <= 0.0 || frame.w == 0 || frame.h == 0 {
        return;
    }

    let uv = egui::Rect::from_min_max(
        egui::pos2(frame.x as f32 / aw, frame.y as f32 / ah),
        egui::pos2(
            (frame.x + frame.w) as f32 / aw,
            (frame.y + frame.h) as f32 / ah,
        ),
    );

    let aspect = frame.w as f32 / frame.h as f32;
    let (tw, th) = if aspect >= 1.0 {
        (THUMB_SIZE, (THUMB_SIZE / aspect).max(1.0))
    } else {
        ((THUMB_SIZE * aspect).max(1.0), THUMB_SIZE)
    };
    let thumb_rect = egui::Rect::from_min_size(
        egui::pos2(
            rect.min.x + (THUMB_SIZE - tw) * 0.5,
            rect.min.y + (THUMB_SIZE - th) * 0.5,
        ),
        egui::vec2(tw, th),
    );

    ui.painter().image(
        atlas_textures[si].id(),
        thumb_rect,
        uv,
        egui::Color32::WHITE,
    );
}

fn show_nodes(
    ui: &mut egui::Ui,
    nodes: &[TreeNode],
    frames: &[FrameInfo],
    sheets: &[SheetData],
    atlas_textures: &[egui::TextureHandle],
    selected: &mut Option<usize>,
) {
    for node in nodes {
        match node {
            TreeNode::Sprite { name, frame_idx } => {
                let idx = *frame_idx;
                let frame = &frames[idx];
                let is_selected = *selected == Some(idx);

                let resp = ui.horizontal(|ui| {
                    draw_thumbnail(ui, frame, sheets, atlas_textures);
                    ui.selectable_label(is_selected, egui::RichText::new(name).small())
                });

                if resp.inner.clicked() {
                    *selected = if is_selected { None } else { Some(idx) };
                }
            }
            TreeNode::Folder {
                name,
                full_path,
                children,
            } => {
                egui::CollapsingHeader::new(egui::RichText::new(name).small().strong())
                    .id_salt(full_path.as_str())
                    .default_open(true)
                    .show(ui, |ui| {
                        show_nodes(ui, children, frames, sheets, atlas_textures, selected);
                    });
            }
        }
    }
}

/// Render the sprite list panel with collapsible folder tree and sprite thumbnails.
pub fn show(ui: &mut egui::Ui, state: &mut AppState, atlas_textures: &[egui::TextureHandle]) {
    ui.horizontal(|ui| {
        ui.strong(t!("sprite_list.sources"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(t!("sprite_list.add")).clicked() {
                state.pending.add_source = true;
            }
        });
    });
    ui.separator();

    let mut remove_idx: Option<usize> = None;
    for (i, source) in state.project.sources.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui.small_button(t!("sprite_list.remove")).clicked() {
                remove_idx = Some(i);
            }
            let full = source.path.to_string_lossy();
            let display = source
                .path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| full.to_string());
            ui.label(egui::RichText::new(display).small())
                .on_hover_text(full.as_ref());
        });
    }
    if state.project.sources.is_empty() {
        ui.label(
            egui::RichText::new(t!("sprite_list.no_sources"))
                .weak()
                .small(),
        );
    }
    if let Some(i) = remove_idx {
        state.remove_source(i);
    }

    ui.add_space(6.0);

    let frame_count = state.frames.len();
    ui.horizontal(|ui| {
        ui.strong(t!("sprite_list.frames", count = frame_count));
    });
    ui.separator();

    if state.frames.is_empty() {
        ui.label(
            egui::RichText::new(t!("sprite_list.pack_hint"))
                .weak()
                .small(),
        );
        return;
    }

    let tree = build_tree(&state.frames);
    let mut new_selected = state.selected_frame;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            show_nodes(
                ui,
                &tree,
                &state.frames,
                &state.sheets,
                atlas_textures,
                &mut new_selected,
            );
        });

    state.selected_frame = new_selected;

    show_sprite_detail(ui, state);
}

fn show_sprite_detail(ui: &mut egui::Ui, state: &mut AppState) {
    let Some(sel_idx) = state.selected_frame else {
        return;
    };
    let Some(frame) = state.frames.get(sel_idx) else {
        return;
    };

    let frame_id = frame.id.clone();
    let frame_w = frame.w;
    let frame_h = frame.h;
    let frame_x = frame.x;
    let frame_y = frame.y;

    ui.separator();
    ui.label(egui::RichText::new(&frame_id).small().strong());
    ui.label(
        egui::RichText::new(format!(
            "{}×{}  ({}, {})",
            frame_w, frame_h, frame_x, frame_y
        ))
        .small()
        .weak(),
    );

    let ovr_idx = state
        .project
        .config
        .sprite_overrides
        .iter()
        .position(|o| o.id == frame_id);

    if let Some(idx) = ovr_idx {
        let (np_chg, pv_chg) = {
            let ovr = &mut state.project.config.sprite_overrides[idx];
            let np =
                crate::widgets::nine_patch_editor::show(ui, &mut ovr.nine_patch, frame_w, frame_h);
            let pv = crate::widgets::pivot_editor::show(ui, &mut ovr.pivot);
            (np, pv)
        };
        if np_chg || pv_chg {
            state.dirty = true;
        }
        if ui.small_button(t!("sprite_list.remove_override")).clicked() {
            state.project.config.sprite_overrides.remove(idx);
            state.dirty = true;
        }
    } else if ui.small_button(t!("sprite_list.add_override")).clicked() {
        state.project.config.sprite_overrides.push(SpriteOverride {
            id: frame_id,
            pivot: None,
            nine_patch: None,
        });
        state.dirty = true;
    }
}
