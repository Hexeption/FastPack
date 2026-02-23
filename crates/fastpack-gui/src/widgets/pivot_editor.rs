use eframe::egui;
use fastpack_core::types::rect::Point;

pub fn show(ui: &mut egui::Ui, pivot: &mut Option<Point>) -> bool {
    let mut changed = false;

    let mut enabled = pivot.is_some();
    if ui.checkbox(&mut enabled, "Custom pivot").changed() {
        *pivot = if enabled {
            Some(Point { x: 0.5, y: 0.5 })
        } else {
            None
        };
        changed = true;
    }

    let Some(pt) = pivot else {
        return changed;
    };

    let preview_size = egui::vec2(80.0, 80.0);
    let (response, painter) = ui.allocate_painter(preview_size, egui::Sense::click_and_drag());
    let rect = response.rect;

    draw_checker(&painter, rect);
    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_gray(100)));

    let cx = rect.min.x + pt.x * rect.width();
    let cy = rect.min.y + pt.y * rect.height();
    let cross_half = 6.0_f32;
    let cross_color = egui::Color32::from_rgb(255, 80, 80);
    let cross_stroke = egui::Stroke::new(1.5, cross_color);
    painter.hline((cx - cross_half)..=(cx + cross_half), cy, cross_stroke);
    painter.vline(cx, (cy - cross_half)..=(cy + cross_half), cross_stroke);
    painter.circle_stroke(egui::pos2(cx, cy), 3.0, cross_stroke);

    if response.dragged() {
        let pos = response.interact_pointer_pos().unwrap_or(egui::pos2(cx, cy));
        pt.x = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
        pt.y = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
        changed = true;
    }

    egui::Grid::new("pivot_values")
        .num_columns(2)
        .spacing([4.0, 4.0])
        .show(ui, |ui| {
            ui.label("X");
            if ui
                .add(egui::Slider::new(&mut pt.x, 0.0..=1.0).fixed_decimals(2))
                .changed()
            {
                changed = true;
            }
            ui.end_row();
            ui.label("Y");
            if ui
                .add(egui::Slider::new(&mut pt.y, 0.0..=1.0).fixed_decimals(2))
                .changed()
            {
                changed = true;
            }
            ui.end_row();
        });

    changed
}

fn draw_checker(painter: &egui::Painter, rect: egui::Rect) {
    let tile = 6.0_f32;
    let c1 = egui::Color32::from_gray(55);
    let c2 = egui::Color32::from_gray(75);
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
