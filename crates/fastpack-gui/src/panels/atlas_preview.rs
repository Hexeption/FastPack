use eframe::egui;

use crate::state::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, atlas: Option<&egui::TextureHandle>) {
    let Some(atlas) = atlas else {
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
    };

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

    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(35, 35, 35));
    draw_checker(&painter, rect);

    let atlas_size = atlas.size();
    let img_w = atlas_size[0] as f32 * zoom;
    let img_h = atlas_size[1] as f32 * zoom;
    let img_origin = rect.center() + pan - egui::vec2(img_w * 0.5, img_h * 0.5);
    let img_rect = egui::Rect::from_min_size(img_origin, egui::vec2(img_w, img_h));

    painter.image(
        atlas.id(),
        img_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    if let Some(idx) = state.selected_frame {
        if let Some(frame) = state.frames.get(idx) {
            let fx = img_origin.x + frame.x as f32 * zoom;
            let fy = img_origin.y + frame.y as f32 * zoom;
            let fw = frame.w as f32 * zoom;
            let fh = frame.h as f32 * zoom;
            let frame_rect = egui::Rect::from_min_size(egui::pos2(fx, fy), egui::vec2(fw, fh));
            painter.rect_stroke(
                frame_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
            );
        }
    }

    if let Some(sheet) = state.sheets.get(state.current_sheet) {
        let text = format!(
            "{}×{}   {} sprites   {} aliases   {} overflow   {:.0}%",
            sheet.width,
            sheet.height,
            state.sprite_count,
            state.alias_count,
            state.overflow_count,
            zoom * 100.0
        );
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
