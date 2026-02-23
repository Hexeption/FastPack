use eframe::egui;

use crate::state::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.strong("Sources");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("Add…").clicked() {
                state.pending.add_source = true;
            }
        });
    });
    ui.separator();

    let mut remove_idx: Option<usize> = None;
    for (i, source) in state.project.sources.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui.small_button("×").clicked() {
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
        ui.label(egui::RichText::new("No sources — add a folder.").weak().small());
    }
    if let Some(i) = remove_idx {
        state.remove_source(i);
    }

    ui.add_space(6.0);

    let frame_count = state.frames.len();
    ui.horizontal(|ui| {
        ui.strong(format!("Frames ({})", frame_count));
    });
    ui.separator();

    if state.frames.is_empty() {
        ui.label(egui::RichText::new("Pack to see sprites.").weak().small());
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, frame) in state.frames.iter().enumerate() {
            let selected = state.selected_frame == Some(i);
            let label = if let Some(ref alias_target) = frame.alias_of {
                format!("{} → {alias_target}", frame.id)
            } else {
                frame.id.clone()
            };
            let response =
                ui.selectable_label(selected, egui::RichText::new(&label).small());
            if response.clicked() {
                state.selected_frame = if selected { None } else { Some(i) };
            }
        }
    });
}
