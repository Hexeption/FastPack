use eframe::egui;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(78, 201, 176);

pub fn apply(ctx: &egui::Context, dark_mode: bool) {
    ctx.set_visuals(if dark_mode {
        custom_dark()
    } else {
        custom_light()
    });
}

fn custom_dark() -> egui::Visuals {
    let mut v = egui::Visuals::dark();

    v.panel_fill = egui::Color32::from_rgb(43, 43, 43);
    v.window_fill = egui::Color32::from_rgb(49, 49, 49);
    v.faint_bg_color = egui::Color32::from_rgb(46, 46, 46);
    v.extreme_bg_color = egui::Color32::from_rgb(26, 26, 26);

    v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(78, 201, 176, 70);
    v.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;

    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(56, 56, 56);
    v.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80));
    v.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220));

    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 60, 60);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(85, 85, 85));
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220));

    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(74, 74, 74);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);

    v.widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(78, 201, 176, 55);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, ACCENT);

    v.widgets.open.bg_fill = egui::Color32::from_rgb(68, 68, 68);
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, ACCENT);

    v
}

fn custom_light() -> egui::Visuals {
    let mut v = egui::Visuals::light();

    v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(78, 201, 176, 100);
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 160, 160));
    v.hyperlink_color = egui::Color32::from_rgb(0, 140, 140);

    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 160, 160));
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 140, 140));

    v
}
