use eframe::egui;
use egui_phosphor::regular as ph;

use crate::state::AppState;

pub fn show(ctx: &egui::Context, state: &mut AppState, atlas_textures: &[egui::TextureHandle]) {
    if !state.anim_preview.open {
        return;
    }

    let n = state.selected_frames.len();
    if n == 0 {
        state.anim_preview.open = false;
        return;
    }

    if state.anim_preview.playing {
        let dt = ctx.input(|i| i.unstable_dt) as f64;
        state.anim_preview.elapsed_secs += dt;
        let frame_dur = 1.0 / state.anim_preview.fps as f64;
        while state.anim_preview.elapsed_secs >= frame_dur {
            state.anim_preview.elapsed_secs -= frame_dur;
            let next = state.anim_preview.current_frame + 1;
            if next >= n {
                if state.anim_preview.looping {
                    state.anim_preview.current_frame = 0;
                } else {
                    state.anim_preview.current_frame = n - 1;
                    state.anim_preview.playing = false;
                }
            } else {
                state.anim_preview.current_frame = next;
            }
        }
        ctx.request_repaint();
    }

    if state.anim_preview.current_frame >= n {
        state.anim_preview.current_frame = n - 1;
    }

    let mut open = state.anim_preview.open;
    egui::Window::new("Animation Preview")
        .open(&mut open)
        .resizable(true)
        .default_size([320.0, 400.0])
        .min_size([240.0, 200.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(ph::CARET_LEFT).clicked() {
                    state.anim_preview.playing = false;
                    state.anim_preview.current_frame =
                        state.anim_preview.current_frame.saturating_sub(1);
                    state.anim_preview.elapsed_secs = 0.0;
                }

                let play_label = if state.anim_preview.playing {
                    ph::PAUSE
                } else {
                    ph::PLAY
                };
                if ui.button(play_label).clicked() {
                    state.anim_preview.playing = !state.anim_preview.playing;
                    state.anim_preview.elapsed_secs = 0.0;
                }

                if ui.button(ph::CARET_RIGHT).clicked() {
                    state.anim_preview.playing = false;
                    let next = state.anim_preview.current_frame + 1;
                    state.anim_preview.current_frame = if next >= n { 0 } else { next };
                    state.anim_preview.elapsed_secs = 0.0;
                }

                ui.separator();

                ui.label("FPS");
                ui.add(egui::Slider::new(&mut state.anim_preview.fps, 1.0..=60.0).step_by(1.0));

                ui.separator();

                ui.label(format!("{} / {}", state.anim_preview.current_frame + 1, n));

                ui.separator();

                ui.checkbox(&mut state.anim_preview.looping, "Loop");
            });

            ui.separator();

            let available = ui.available_size();
            let (response, painter) = ui.allocate_painter(available, egui::Sense::click_and_drag());

            if response.dragged() {
                let d = response.drag_delta();
                state.anim_preview.pan[0] += d.x;
                state.anim_preview.pan[1] += d.y;
            }

            let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
            if response.hovered() && scroll_y != 0.0 {
                let factor: f32 = if scroll_y > 0.0 { 1.1 } else { 1.0 / 1.1 };
                state.anim_preview.zoom = (state.anim_preview.zoom * factor).clamp(0.05, 64.0);
            }

            if response.double_clicked() {
                state.anim_preview.pan = [0.0, 0.0];
                state.anim_preview.zoom = 1.0;
            }

            let canvas = response.rect;
            let zoom = state.anim_preview.zoom;
            let pan = egui::vec2(state.anim_preview.pan[0], state.anim_preview.pan[1]);

            painter.rect_filled(canvas, 0.0, egui::Color32::from_rgb(24, 24, 24));
            draw_checker(&painter, canvas);

            let frame_idx = state.selected_frames[state.anim_preview.current_frame];
            if let Some(frame) = state.frames.get(frame_idx) {
                let si = frame.sheet_idx;
                if si < atlas_textures.len() && si < state.sheets.len() {
                    let sheet = &state.sheets[si];
                    let aw = sheet.width as f32;
                    let ah = sheet.height as f32;
                    let uv = egui::Rect::from_min_max(
                        egui::pos2(frame.x as f32 / aw, frame.y as f32 / ah),
                        egui::pos2(
                            (frame.x + frame.w) as f32 / aw,
                            (frame.y + frame.h) as f32 / ah,
                        ),
                    );

                    let sw = frame.w as f32 * zoom;
                    let sh = frame.h as f32 * zoom;
                    let origin = canvas.center() + pan - egui::vec2(sw * 0.5, sh * 0.5);
                    let img_rect = egui::Rect::from_min_size(origin, egui::vec2(sw, sh));

                    painter.image(atlas_textures[si].id(), img_rect, uv, egui::Color32::WHITE);

                    painter.text(
                        canvas.left_bottom() + egui::vec2(6.0, -6.0),
                        egui::Align2::LEFT_BOTTOM,
                        &frame.id,
                        egui::FontId::proportional(11.0),
                        egui::Color32::WHITE,
                    );
                }
            }
        });

    state.anim_preview.open = open;
}

fn draw_checker(painter: &egui::Painter, rect: egui::Rect) {
    let tile = 8.0_f32;
    let c1 = egui::Color32::from_rgb(32, 32, 32);
    let c2 = egui::Color32::from_rgb(40, 40, 40);
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
