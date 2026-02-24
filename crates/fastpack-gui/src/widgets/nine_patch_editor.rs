use eframe::egui;
use fastpack_core::types::sprite::NinePatch;
use rust_i18n::t;

pub fn show(
    ui: &mut egui::Ui,
    nine_patch: &mut Option<NinePatch>,
    sprite_w: u32,
    sprite_h: u32,
) -> bool {
    let mut changed = false;

    let mut enabled = nine_patch.is_some();
    if ui
        .checkbox(&mut enabled, t!("widgets.nine_patch"))
        .changed()
    {
        *nine_patch = if enabled {
            Some(NinePatch {
                top: 0,
                right: 0,
                bottom: 0,
                left: 0,
            })
        } else {
            None
        };
        changed = true;
    }

    let Some(np) = nine_patch else {
        return changed;
    };

    let preview_size = egui::vec2(64.0, 64.0);
    let (_, painter) = ui.allocate_painter(preview_size, egui::Sense::hover());
    let rect = painter.clip_rect();
    draw_checker(&painter, rect);

    let w = sprite_w.max(1) as f32;
    let h = sprite_h.max(1) as f32;
    let sx = rect.width() / w;
    let sy = rect.height() / h;
    let color = egui::Color32::from_rgb(80, 180, 255);
    let stroke = egui::Stroke::new(1.0, color);

    painter.hline(
        rect.min.x..=rect.max.x,
        rect.min.y + np.top as f32 * sy,
        stroke,
    );
    painter.hline(
        rect.min.x..=rect.max.x,
        rect.max.y - np.bottom as f32 * sy,
        stroke,
    );
    painter.vline(
        rect.min.x + np.left as f32 * sx,
        rect.min.y..=rect.max.y,
        stroke,
    );
    painter.vline(
        rect.max.x - np.right as f32 * sx,
        rect.min.y..=rect.max.y,
        stroke,
    );
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
    );

    egui::Grid::new("nine_patch_values")
        .num_columns(4)
        .spacing([4.0, 4.0])
        .show(ui, |ui| {
            ui.label(t!("widgets.nine_patch_t"));
            if ui
                .add(egui::DragValue::new(&mut np.top).range(0..=sprite_h))
                .changed()
            {
                changed = true;
            }
            ui.label(t!("widgets.nine_patch_b"));
            if ui
                .add(egui::DragValue::new(&mut np.bottom).range(0..=sprite_h))
                .changed()
            {
                changed = true;
            }
            ui.end_row();
            ui.label(t!("widgets.nine_patch_l"));
            if ui
                .add(egui::DragValue::new(&mut np.left).range(0..=sprite_w))
                .changed()
            {
                changed = true;
            }
            ui.label(t!("widgets.nine_patch_r"));
            if ui
                .add(egui::DragValue::new(&mut np.right).range(0..=sprite_w))
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
