use eframe::egui;

use crate::state::AppState;

const GAP: f32 = 16.0;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, atlases: &[egui::TextureHandle]) {
    if atlases.is_empty() {
        if state.packing {
            ui.centered_and_justified(|ui| {
                ui.spinner();
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Pack sprites to see the atlas.");
            });
        }
        return;
    }

    let available = ui.available_size();
    let (response, painter) = ui.allocate_painter(available, egui::Sense::click_and_drag());

    if response.dragged() {
        let d = response.drag_delta();
        state.atlas_pan[0] += d.x;
        state.atlas_pan[1] += d.y;
    }

    let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
    if response.hovered() && scroll_y != 0.0 {
        let factor: f32 = if scroll_y > 0.0 { 1.04 } else { 1.0 / 1.04 };
        state.atlas_zoom = (state.atlas_zoom * factor).clamp(0.05, 64.0);
    }

    if response.double_clicked() {
        state.atlas_pan = [0.0, 0.0];
        state.atlas_zoom = 1.0;
    }

    let rect = response.rect;
    let zoom = state.atlas_zoom;
    let pan = egui::vec2(state.atlas_pan[0], state.atlas_pan[1]);
    let n = atlases.len();

    let total_atlas_w: f32 =
        state.sheets.iter().map(|s| s.width as f32).sum::<f32>() + (n as f32 - 1.0) * GAP;
    let max_atlas_h: f32 = state
        .sheets
        .iter()
        .map(|s| s.height as f32)
        .fold(0.0_f32, f32::max);

    let group_origin =
        rect.center() + pan - egui::vec2(total_atlas_w * zoom * 0.5, max_atlas_h * zoom * 0.5);

    let sheet_origins: Vec<egui::Pos2> = {
        let mut cx = 0.0_f32;
        state
            .sheets
            .iter()
            .map(|sheet| {
                let x = group_origin.x + cx;
                let y = group_origin.y + (max_atlas_h - sheet.height as f32) * zoom * 0.5;
                cx += (sheet.width as f32 + GAP) * zoom;
                egui::pos2(x, y)
            })
            .collect()
    };

    let frame_offsets: Vec<usize> = {
        let mut off = 0;
        state
            .sheets
            .iter()
            .map(|s| {
                let o = off;
                off += s.frames.len();
                o
            })
            .collect()
    };

    // Click to select a sprite
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let mut hit = None;
            'search: for (si, sheet) in state.sheets.iter().enumerate() {
                let origin = sheet_origins[si];
                let ax = ((pos.x - origin.x) / zoom) as i32;
                let ay = ((pos.y - origin.y) / zoom) as i32;
                for (fi, f) in sheet.frames.iter().enumerate() {
                    if ax >= f.x as i32
                        && ax < (f.x + f.w) as i32
                        && ay >= f.y as i32
                        && ay < (f.y + f.h) as i32
                    {
                        hit = Some(frame_offsets[si] + fi);
                        break 'search;
                    }
                }
            }
            state.selected_frame = hit;
        }
    }

    // Dark panel background
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(35, 35, 35));

    let full_uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    let selected_frame = state.selected_frame;

    for (i, (sheet, texture)) in state.sheets.iter().zip(atlases.iter()).enumerate() {
        let origin = sheet_origins[i];
        let iw = sheet.width as f32 * zoom;
        let ih = sheet.height as f32 * zoom;
        let img_rect = egui::Rect::from_min_size(origin, egui::vec2(iw, ih));
        let atlas_w = sheet.width as f32;
        let atlas_h = sheet.height as f32;

        // Checkerboard within this sheet's bounds
        let checker_rect = img_rect.intersect(rect);
        if checker_rect.is_positive() {
            draw_checker(&painter, checker_rect);
        }

        // Is the selected frame on this sheet?
        let sel_local = selected_frame
            .filter(|&g| g >= frame_offsets[i] && g < frame_offsets[i] + sheet.frames.len())
            .map(|g| g - frame_offsets[i]);

        if let Some(local_idx) = sel_local {
            if let Some(frame) = sheet.frames.get(local_idx) {
                // Draw full atlas dimmed
                painter.image(
                    texture.id(),
                    img_rect,
                    full_uv,
                    egui::Color32::from_rgb(60, 60, 60),
                );

                // Draw selected frame at full brightness
                let fx = origin.x + frame.x as f32 * zoom;
                let fy = origin.y + frame.y as f32 * zoom;
                let frame_rect = egui::Rect::from_min_size(
                    egui::pos2(fx, fy),
                    egui::vec2(frame.w as f32 * zoom, frame.h as f32 * zoom),
                );
                let uv = egui::Rect::from_min_max(
                    egui::pos2(frame.x as f32 / atlas_w, frame.y as f32 / atlas_h),
                    egui::pos2(
                        (frame.x + frame.w) as f32 / atlas_w,
                        (frame.y + frame.h) as f32 / atlas_h,
                    ),
                );
                painter.image(texture.id(), frame_rect, uv, egui::Color32::WHITE);

                // Yellow highlight border
                painter.rect_stroke(
                    frame_rect,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                );
            } else {
                painter.image(texture.id(), img_rect, full_uv, egui::Color32::WHITE);
            }
        } else {
            painter.image(texture.id(), img_rect, full_uv, egui::Color32::WHITE);
        }

        // Per-sheet label when multiple sheets are present
        if n > 1 {
            painter.text(
                egui::pos2(origin.x + 4.0, origin.y + ih - 4.0),
                egui::Align2::LEFT_BOTTOM,
                format!("Sheet {}: {}×{}", i + 1, sheet.width, sheet.height),
                egui::FontId::proportional(10.0),
                egui::Color32::WHITE,
            );
        }
    }

    // Stats label at bottom-left of the panel
    let stats_text = if n == 1 {
        state.sheets.first().map(|sheet| {
            format!(
                "{}×{}   {} sprites   {} aliases   {} overflow   {:.0}%",
                sheet.width,
                sheet.height,
                state.sprite_count,
                state.alias_count,
                state.overflow_count,
                zoom * 100.0
            )
        })
    } else {
        Some(format!(
            "{} sprites   {} aliases   {} overflow   {:.0}%",
            state.sprite_count,
            state.alias_count,
            state.overflow_count,
            zoom * 100.0
        ))
    };
    if let Some(text) = stats_text {
        painter.text(
            rect.left_bottom() + egui::vec2(6.0, -6.0),
            egui::Align2::LEFT_BOTTOM,
            text,
            egui::FontId::proportional(11.0),
            egui::Color32::WHITE,
        );
    }
}

fn draw_checker(painter: &egui::Painter, rect: egui::Rect) {
    let tile = 8.0_f32;
    let c1 = egui::Color32::from_rgb(50, 50, 50);
    let c2 = egui::Color32::from_rgb(60, 60, 60);
    let mut x = (rect.min.x / tile).floor() * tile;
    while x < rect.max.x {
        let mut y = (rect.min.y / tile).floor() * tile;
        while y < rect.max.y {
            let chess = ((x / tile) as i32 + (y / tile) as i32) % 2 == 0;
            let color = if chess { c1 } else { c2 };
            let tile_rect = egui::Rect::from_min_max(
                egui::pos2(x.max(rect.min.x), y.max(rect.min.y)),
                egui::pos2((x + tile).min(rect.max.x), (y + tile).min(rect.max.y)),
            );
            painter.rect_filled(tile_rect, 0.0, color);
            y += tile;
        }
        x += tile;
    }
}
