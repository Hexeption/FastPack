use eframe::egui;
use fastpack_core::types::config::SpriteOverride;
use rust_i18n::t;

use crate::state::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
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

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, frame) in state.frames.iter().enumerate() {
                let selected = state.selected_frame == Some(i);
                let response =
                    ui.selectable_label(selected, egui::RichText::new(&frame.id).small());
                if response.clicked() {
                    state.selected_frame = if selected { None } else { Some(i) };
                }
            }
        });

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
