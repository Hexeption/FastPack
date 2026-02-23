use eframe::egui;

use crate::{
    menu,
    panels::{atlas_preview, output_log, settings, sprite_list},
    state::AppState,
    toolbar,
};

/// The top-level eframe application struct.
pub struct FastPackApp {
    pub state: AppState,
}

impl Default for FastPackApp {
    fn default() -> Self {
        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for FastPackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            menu::show(ui, &mut self.state);
        });

        // Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar::show(ui, &mut self.state);
        });

        // Output log at the bottom
        egui::TopBottomPanel::bottom("output_log").show(ctx, |ui| {
            output_log::show(ui, &mut self.state);
        });

        // Sprite list on the left
        egui::SidePanel::left("sprite_list").show(ctx, |ui| {
            sprite_list::show(ui, &mut self.state);
        });

        // Settings on the right
        egui::SidePanel::right("settings").show(ctx, |ui| {
            settings::show(ui, &mut self.state);
        });

        // Atlas preview fills the centre
        egui::CentralPanel::default().show(ctx, |ui| {
            atlas_preview::show(ui, &mut self.state);
        });
    }
}
