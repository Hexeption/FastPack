use eframe::egui;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(62, 176, 154);

pub fn apply(ctx: &egui::Context, dark_mode: bool) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 3.0);
    style.spacing.window_margin = egui::Margin::same(6);
    style.interaction.selectable_labels = false;
    ctx.set_style(style);

    ctx.set_visuals(if dark_mode {
        custom_dark()
    } else {
        custom_light()
    });
}

fn custom_dark() -> egui::Visuals {
    let mut v = egui::Visuals::dark();

    v.panel_fill = egui::Color32::from_rgb(30, 30, 30);
    v.window_fill = egui::Color32::from_rgb(36, 36, 36);
    v.faint_bg_color = egui::Color32::from_rgb(34, 34, 34);
    v.extreme_bg_color = egui::Color32::from_rgb(22, 22, 22);

    v.window_corner_radius = egui::CornerRadius::same(4);

    v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(62, 176, 154, 100);
    v.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;

    let corner = egui::CornerRadius::same(3);
    let separator = egui::Color32::from_rgb(55, 55, 55);

    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(38, 38, 38);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, separator);
    v.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200));
    v.widgets.noninteractive.corner_radius = corner;

    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(48, 48, 48);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(65, 65, 65));
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200));
    v.widgets.inactive.corner_radius = corner;

    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(58, 58, 58);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);
    v.widgets.hovered.corner_radius = corner;

    v.widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(62, 176, 154, 55);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, ACCENT);
    v.widgets.active.corner_radius = corner;

    v.widgets.open.bg_fill = egui::Color32::from_rgb(50, 50, 50);
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, ACCENT);
    v.widgets.open.corner_radius = corner;

    v
}

fn custom_light() -> egui::Visuals {
    let mut v = egui::Visuals::light();

    v.panel_fill = egui::Color32::from_rgb(238, 238, 242);
    v.window_fill = egui::Color32::from_rgb(246, 246, 250);
    v.faint_bg_color = egui::Color32::from_rgb(228, 228, 233);
    v.extreme_bg_color = egui::Color32::from_rgb(255, 255, 255);

    v.window_corner_radius = egui::CornerRadius::same(4);

    let accent = egui::Color32::from_rgb(0, 155, 135);
    v.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(62, 176, 154, 70);
    v.selection.stroke = egui::Stroke::new(1.0, accent);
    v.hyperlink_color = accent;

    let corner = egui::CornerRadius::same(3);
    let separator = egui::Color32::from_rgb(205, 205, 210);

    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(228, 228, 233);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, separator);
    v.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 55, 62));
    v.widgets.noninteractive.corner_radius = corner;

    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(218, 218, 224);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(192, 192, 200));
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 45, 52));
    v.widgets.inactive.corner_radius = corner;

    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(206, 206, 214);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 30, 36));
    v.widgets.hovered.corner_radius = corner;

    v.widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(62, 176, 154, 45);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, accent);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(20, 20, 26));
    v.widgets.active.corner_radius = corner;

    v.widgets.open.bg_fill = egui::Color32::from_rgb(212, 212, 220);
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, accent);
    v.widgets.open.corner_radius = corner;

    v
}
