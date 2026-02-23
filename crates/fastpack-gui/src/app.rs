use std::sync::mpsc;

use eframe::egui;

use crate::{
    menu,
    panels::{atlas_preview, output_log, settings, sprite_list},
    state::AppState,
    toolbar,
    worker::{WorkerMessage, run_pack},
};

#[derive(Default)]
pub struct FastPackApp {
    pub state: AppState,
    pub atlas_texture: Option<egui::TextureHandle>,
    worker_rx: Option<mpsc::Receiver<WorkerMessage>>,
}

impl eframe::App for FastPackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::theme::apply(ctx, self.state.dark_mode);
        self.poll_worker(ctx);
        self.handle_pending(ctx);
        self.handle_dropped_files(ctx);

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.state.window_title()));

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            menu::show(ui, &mut self.state);
        });

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar::show(ui, &mut self.state);
        });

        egui::TopBottomPanel::bottom("output_log")
            .min_height(80.0)
            .resizable(true)
            .show(ctx, |ui| {
                output_log::show(ui, &mut self.state);
            });

        egui::SidePanel::left("sprite_list")
            .min_width(160.0)
            .resizable(true)
            .show(ctx, |ui| {
                sprite_list::show(ui, &mut self.state);
            });

        egui::SidePanel::right("settings")
            .min_width(220.0)
            .resizable(true)
            .show(ctx, |ui| {
                settings::show(ui, &mut self.state);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            atlas_preview::show(ui, &mut self.state, self.atlas_texture.as_ref());

            let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
            if hovering {
                let overlay_rect = ui.max_rect();
                ui.painter().rect_filled(
                    overlay_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(20, 80, 160, 120),
                );
                ui.painter().text(
                    overlay_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Drop folders or .fpsheet here",
                    egui::FontId::proportional(18.0),
                    egui::Color32::WHITE,
                );
            }
        });
    }
}

impl FastPackApp {
    fn poll_worker(&mut self, ctx: &egui::Context) {
        let mut finished = false;
        if let Some(rx) = &self.worker_rx {
            loop {
                match rx.try_recv() {
                    Ok(WorkerMessage::Started) => {
                        self.state.packing = true;
                    }
                    Ok(WorkerMessage::Progress { .. }) => {}
                    Ok(WorkerMessage::Finished(output)) => {
                        self.state.packing = false;
                        self.state.sprite_count = output.sprite_count;
                        self.state.alias_count = output.alias_count;
                        self.state.overflow_count = output.overflow_count;
                        self.state.frames = output
                            .frames
                            .into_iter()
                            .map(|f| crate::state::FrameInfo {
                                id: f.id,
                                x: f.x,
                                y: f.y,
                                w: f.w,
                                h: f.h,
                                alias_of: f.alias_of,
                            })
                            .collect();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [output.width as usize, output.height as usize],
                            &output.rgba,
                        );
                        self.atlas_texture = Some(ctx.load_texture(
                            "atlas",
                            color_image,
                            egui::TextureOptions::default(),
                        ));
                        self.state.atlas_rgba = Some((output.rgba, output.width, output.height));
                        self.state.log_info(format!(
                            "Packed {} sprites — {}×{}  ({} aliases, {} overflow)",
                            self.state.sprite_count,
                            output.width,
                            output.height,
                            self.state.alias_count,
                            self.state.overflow_count,
                        ));
                        finished = true;
                    }
                    Ok(WorkerMessage::Failed(msg)) => {
                        self.state.packing = false;
                        self.state.log_error(format!("Pack failed: {msg}"));
                        finished = true;
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        finished = true;
                        break;
                    }
                }
            }
        }
        if finished {
            self.worker_rx = None;
        }
    }

    fn handle_pending(&mut self, ctx: &egui::Context) {
        if std::mem::take(&mut self.state.pending.pack) {
            self.spawn_pack(ctx.clone());
        }
        if std::mem::take(&mut self.state.pending.new_project) {
            self.state.new_project();
            self.atlas_texture = None;
        }
        if std::mem::take(&mut self.state.pending.open_project) {
            self.do_open_project();
        }
        if std::mem::take(&mut self.state.pending.save_project) {
            self.do_save_project(false);
        }
        if std::mem::take(&mut self.state.pending.save_project_as) {
            self.do_save_project(true);
        }
        if std::mem::take(&mut self.state.pending.add_source) {
            self.do_add_source();
        }
    }

    fn spawn_pack(&mut self, ctx: egui::Context) {
        if self.state.packing {
            return;
        }
        if self.state.project.sources.is_empty() {
            self.state
                .log_warn("No source directories configured. Add sprites first.");
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.worker_rx = Some(rx);
        let project = self.state.project.clone();
        std::thread::spawn(move || {
            tx.send(WorkerMessage::Started).ok();
            match run_pack(&project) {
                Ok(output) => {
                    tx.send(WorkerMessage::Finished(Box::new(output))).ok();
                }
                Err(e) => {
                    tx.send(WorkerMessage::Failed(e.to_string())).ok();
                }
            }
            ctx.request_repaint();
        });
    }

    fn do_open_project(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("FastPack Project", &["fpsheet"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => match toml::from_str(&text) {
                Ok(project) => {
                    self.state.project = project;
                    self.state.project_path = Some(path.clone());
                    self.state.dirty = false;
                    self.state.frames.clear();
                    self.atlas_texture = None;
                    self.state.log_info(format!("Opened {}", path.display()));
                }
                Err(e) => self
                    .state
                    .log_error(format!("Failed to parse project: {e}")),
            },
            Err(e) => self.state.log_error(format!("Failed to read project: {e}")),
        }
    }

    fn do_save_project(&mut self, force_dialog: bool) {
        let path = if force_dialog || self.state.project_path.is_none() {
            rfd::FileDialog::new()
                .set_file_name("project.fpsheet")
                .add_filter("FastPack Project", &["fpsheet"])
                .save_file()
        } else {
            self.state.project_path.clone()
        };
        let Some(path) = path else { return };
        match toml::to_string_pretty(&self.state.project) {
            Ok(text) => match std::fs::write(&path, text.as_bytes()) {
                Ok(()) => {
                    self.state.project_path = Some(path.clone());
                    self.state.dirty = false;
                    self.state.log_info(format!("Saved {}", path.display()));
                }
                Err(e) => self
                    .state
                    .log_error(format!("Failed to write project: {e}")),
            },
            Err(e) => self
                .state
                .log_error(format!("Failed to serialise project: {e}")),
        }
    }

    fn do_add_source(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.state.add_source_path(path);
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            let Some(path) = file.path else { continue };

            if path.extension().and_then(|e| e.to_str()) == Some("fpsheet") {
                match std::fs::read_to_string(&path) {
                    Ok(text) => match toml::from_str(&text) {
                        Ok(project) => {
                            self.state.project = project;
                            self.state.project_path = Some(path.clone());
                            self.state.dirty = false;
                            self.state.frames.clear();
                            self.atlas_texture = None;
                            self.state.log_info(format!("Opened {}", path.display()));
                        }
                        Err(e) => self
                            .state
                            .log_error(format!("Failed to parse project: {e}")),
                    },
                    Err(e) => self.state.log_error(format!("Failed to read file: {e}")),
                }
            } else if path.is_dir() {
                self.state.add_source_path(path);
            } else if path.is_file() {
                if let Some(parent) = path.parent() {
                    self.state.add_source_path(parent.to_path_buf());
                }
            }
        }
    }
}
