use eframe::egui;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(52, 211, 199);

pub fn apply(ctx: &egui::Context, dark_mode: bool) {
    ctx.set_visuals(if dark_mode {
        custom_dark()
    } else {
        custom_light()
    });
}

fn custom_dark() -> egui::Visuals {
    let mut v = egui::Visuals::dark();

    v.panel_fill = egui::Color32::from_rgb(20, 20, 28);
    v.window_fill = egui::Color32::from_rgb(28, 28, 38);
    v.faint_bg_color = egui::Color32::from_rgb(26, 26, 36);
    v.extreme_bg_color = egui::Color32::from_rgb(10, 10, 15);

    v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(52, 211, 199, 70);
    v.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;

    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(32, 32, 44);
    v.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 55, 72));

    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(38, 38, 52);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 80));

    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(50, 50, 68);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);

    v.widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(52, 211, 199, 55);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, ACCENT);

    v.widgets.open.bg_fill = egui::Color32::from_rgb(44, 44, 60);
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, ACCENT);

    v
}

fn custom_light() -> egui::Visuals {
    let mut v = egui::Visuals::light();

    v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(52, 211, 199, 100);
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 160, 160));
    v.hyperlink_color = egui::Color32::from_rgb(0, 140, 140);

    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 160, 160));
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 140, 140));

    v
}
