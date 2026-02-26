use std::sync::mpsc;
use std::time::Duration;

use eframe::egui;
use fastpack_compress::{
    backends::{jpeg::JpegCompressor, png::PngCompressor, webp::WebpCompressor},
    compressor::{CompressInput, Compressor},
};
use fastpack_core::types::{
    atlas::PackedAtlas,
    config::DataFormat,
    pixel_format::{PixelFormat, TextureFormat},
    rect::Size,
};
use fastpack_formats::{
    exporter::{ExportInput, Exporter},
    formats::{
        json_array::JsonArrayExporter, json_hash::JsonHashExporter, phaser3::Phaser3Exporter,
        pixijs::PixiJsExporter,
    },
};
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};

use crate::{
    menu,
    panels::{anim_preview, atlas_preview, output_log, prefs_window, settings, sprite_list},
    preferences::Preferences,
    state::AppState,
    toolbar,
    updater::{UpdateMsg, UpdateStatus},
    worker::{WorkerMessage, run_pack},
};
use rust_i18n::t;

/// The root application type that implements `eframe::App`.
pub struct FastPackApp {
    /// Shared GUI state and active project data.
    pub state: AppState,
    /// Rendered atlas texture handles for the current pack result.
    pub atlas_textures: Vec<egui::TextureHandle>,
    worker_rx: Option<mpsc::Receiver<WorkerMessage>>,
    /// Persistent user preferences loaded from disk.
    pub prefs: Preferences,
    prefs_open: bool,
    update_status: UpdateStatus,
    update_rx: Option<mpsc::Receiver<UpdateMsg>>,
    file_watcher: Option<Box<dyn Send>>,
    watch_rx: Option<mpsc::Receiver<DebounceEventResult>>,
    /// System DPI pixels-per-point captured at startup; used to apply ui_scale.
    pub native_pixels_per_point: f32,
}

impl Default for FastPackApp {
    fn default() -> Self {
        let prefs = Preferences::load();
        rust_i18n::set_locale(prefs.language.code());
        let state = AppState {
            dark_mode: prefs.dark_mode,
            ..AppState::default()
        };
        let mut app = Self {
            state,
            atlas_textures: Vec::new(),
            worker_rx: None,
            prefs,
            prefs_open: false,
            update_status: UpdateStatus::Idle,
            update_rx: None,
            file_watcher: None,
            watch_rx: None,
            native_pixels_per_point: 0.0,
        };
        if app.prefs.auto_check_updates {
            let (tx, rx) = mpsc::channel();
            crate::updater::spawn_check(tx);
            app.update_rx = Some(rx);
            app.update_status = UpdateStatus::Checking;
        }
        app
    }
}

impl eframe::App for FastPackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::theme::apply(ctx, self.state.dark_mode);
        self.apply_ui_scale(ctx);
        self.poll_worker(ctx);
        self.poll_watcher(ctx);
        self.handle_pending(ctx);
        self.handle_dropped_files(ctx);

        // Sync dark_mode back to prefs when the toolbar toggles it.
        if self.prefs.dark_mode != self.state.dark_mode {
            self.prefs.dark_mode = self.state.dark_mode;
            self.prefs.save();
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.state.window_title()));

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            menu::show(ui, &mut self.state, &self.prefs.keybinds);
        });

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar::show(ui, &mut self.state);
        });

        egui::TopBottomPanel::bottom("output_log")
            .min_height(80.0)
            .default_height(100.0)
            .resizable(true)
            .show(ctx, |ui| {
                output_log::show(ui, &mut self.state);
            });

        egui::SidePanel::left("sprite_list")
            .min_width(180.0)
            .default_width(220.0)
            .resizable(true)
            .show(ctx, |ui| {
                sprite_list::show(ui, &mut self.state, &self.atlas_textures);
            });

        egui::SidePanel::right("settings")
            .min_width(260.0)
            .default_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                settings::show(ui, &mut self.state);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            atlas_preview::show(ui, &mut self.state, &self.atlas_textures);

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
                    t!("drop_overlay"),
                    egui::FontId::proportional(18.0),
                    egui::Color32::WHITE,
                );
            }
        });

        if self.prefs_open {
            prefs_window::show(
                ctx,
                &mut self.prefs,
                &mut self.prefs_open,
                &mut self.update_status,
                &mut self.update_rx,
            );
        }

        anim_preview::show(ctx, &mut self.state, &self.atlas_textures);
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
                        self.state.selected_frames.clear();
                        self.state.anchor_frame = None;
                        self.state.anim_preview.open = false;
                        self.atlas_textures.clear();
                        self.state.sheets.clear();

                        for (sheet_idx, sheet) in output.sheets.into_iter().enumerate() {
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                [sheet.width as usize, sheet.height as usize],
                                &sheet.rgba,
                            );
                            self.atlas_textures.push(ctx.load_texture(
                                "atlas",
                                color_image,
                                egui::TextureOptions::default(),
                            ));
                            let frames: Vec<crate::state::FrameInfo> = sheet
                                .frames
                                .into_iter()
                                .map(|f| crate::state::FrameInfo {
                                    id: f.id,
                                    sheet_idx,
                                    x: f.x,
                                    y: f.y,
                                    w: f.w,
                                    h: f.h,
                                    alias_of: f.alias_of,
                                })
                                .collect();
                            self.state.sheets.push(crate::state::SheetData {
                                rgba: sheet.rgba,
                                width: sheet.width,
                                height: sheet.height,
                                frames,
                                atlas_frames: sheet.atlas_frames,
                            });
                        }

                        self.state.frames = self
                            .state
                            .sheets
                            .iter()
                            .flat_map(|s| s.frames.iter().cloned())
                            .collect();
                        self.state.frames.sort_unstable_by(|a, b| a.id.cmp(&b.id));

                        let sheet_count = self.state.sheets.len();
                        let (w, h) = self
                            .state
                            .sheets
                            .first()
                            .map(|s| (s.width, s.height))
                            .unwrap_or_default();
                        self.state.log_info(t!(
                            "log.pack_result",
                            sprites = self.state.sprite_count,
                            w = w,
                            h = h,
                            sheets = sheet_count,
                            plural = if sheet_count == 1 { "" } else { "s" },
                            aliases = self.state.alias_count,
                            overflow = self.state.overflow_count,
                        ));
                        finished = true;
                    }
                    Ok(WorkerMessage::Failed(msg)) => {
                        self.state.packing = false;
                        self.state.log_error(t!("log.pack_failed", msg = msg));
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
        if std::mem::take(&mut self.state.pending.export) {
            self.do_export();
        }
        if std::mem::take(&mut self.state.pending.new_project) {
            self.state.new_project(self.prefs.default_config.clone());
            self.atlas_textures.clear();
            self.file_watcher = None;
            self.watch_rx = None;
        }
        if std::mem::take(&mut self.state.pending.open_project) {
            self.do_open_project();
            self.state.pending.rebuild_watcher = true;
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
        if std::mem::take(&mut self.state.pending.open_prefs) {
            self.prefs_open = true;
        }
        if std::mem::take(&mut self.state.pending.rebuild_watcher) {
            self.rebuild_watcher();
        }
    }

    fn spawn_pack(&mut self, ctx: egui::Context) {
        if self.state.packing {
            return;
        }
        if self.state.project.sources.is_empty() {
            self.state.frames.clear();
            self.state.sheets.clear();
            self.atlas_textures.clear();
            self.state.selected_frames.clear();
            self.state.anchor_frame = None;
            self.state.log_warn(t!("log.no_sources"));
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
                    self.atlas_textures.clear();
                    self.state
                        .log_info(t!("log.opened", path = path.display().to_string()));
                }
                Err(e) => self
                    .state
                    .log_error(t!("log.parse_failed", err = e.to_string())),
            },
            Err(e) => self
                .state
                .log_error(t!("log.read_failed", err = e.to_string())),
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
                    self.state
                        .log_info(t!("log.saved", path = path.display().to_string()));
                }
                Err(e) => self
                    .state
                    .log_error(t!("log.write_project_failed", err = e.to_string())),
            },
            Err(e) => self
                .state
                .log_error(t!("log.serialize_failed", err = e.to_string())),
        }
    }

    fn do_add_source(&mut self) {
        if let Some(paths) = rfd::FileDialog::new().pick_folders() {
            for path in paths {
                self.state.add_source_path(path);
            }
        }
    }

    fn do_export(&mut self) {
        if self.state.sheets.is_empty() {
            self.state.log_warn(t!("log.nothing_to_export"));
            return;
        }

        let out_cfg = &self.state.project.config.output;
        let out_dir = out_cfg.directory.clone();
        if out_dir.as_os_str().is_empty() {
            self.state.log_warn(t!("log.no_output_dir"));
            return;
        }

        let texture_format = out_cfg.texture_format;
        let pixel_format = out_cfg.pixel_format;
        let quality = out_cfg.quality;
        let data_format = out_cfg.data_format;
        let name = out_cfg.name.clone();
        let pack_mode = self.state.project.config.layout.pack_mode;

        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            self.state
                .log_error(t!("log.create_dir_failed", err = e.to_string()));
            return;
        }

        let compressor: Box<dyn Compressor> = match texture_format {
            TextureFormat::Jpeg => Box::new(JpegCompressor),
            TextureFormat::WebP => Box::new(WebpCompressor),
            _ => Box::new(PngCompressor),
        };

        let pixel_format_str = match pixel_format {
            PixelFormat::Rgba8888 => "RGBA8888",
            PixelFormat::Rgb888 => "RGB888",
            PixelFormat::Rgb565 => "RGB565",
            PixelFormat::Rgba4444 => "RGBA4444",
            PixelFormat::Rgba5551 => "RGBA5551",
            PixelFormat::Alpha8 => "ALPHA8",
        };

        let exporter: Box<dyn Exporter> = match data_format {
            DataFormat::JsonArray => Box::new(JsonArrayExporter),
            DataFormat::Phaser3 => Box::new(Phaser3Exporter),
            DataFormat::Pixijs => Box::new(PixiJsExporter),
            DataFormat::JsonHash => Box::new(JsonHashExporter),
        };

        let sheet_base = |i: usize| -> String {
            if i == 0 {
                name.clone()
            } else {
                format!("{name}{i}")
            }
        };

        let tex_ext = compressor.file_extension();

        // Compress textures and build per-sheet metadata.
        let mut packed_atlases: Vec<PackedAtlas> = Vec::new();
        let mut tex_filenames: Vec<String> = Vec::new();

        for i in 0..self.state.sheets.len() {
            let (width, height, rgba, atlas_frames) = {
                let sheet = &self.state.sheets[i];
                (
                    sheet.width,
                    sheet.height,
                    sheet.rgba.clone(),
                    sheet.atlas_frames.clone(),
                )
            };

            let atlas_image =
                image::RgbaImage::from_raw(width, height, rgba).expect("valid rgba buffer");
            let dyn_image = image::DynamicImage::from(atlas_image);

            let texture_bytes = match compressor.compress(&CompressInput {
                image: &dyn_image,
                pack_mode,
                quality,
            }) {
                Ok(output) => output.data,
                Err(e) => {
                    self.state
                        .log_error(t!("log.compress_failed", i = i, err = e.to_string()));
                    return;
                }
            };

            let tex_filename = format!("{}.{}", sheet_base(i), tex_ext);
            let tex_path = out_dir.join(&tex_filename);

            if let Err(e) = std::fs::write(&tex_path, &texture_bytes) {
                self.state
                    .log_error(t!("log.write_texture_failed", i = i, err = e.to_string()));
                return;
            }

            let tex_kb = texture_bytes.len() as f64 / 1024.0;
            self.state.log_info(t!(
                "log.wrote_texture",
                path = tex_path.display().to_string(),
                kb = format!("{:.1}", tex_kb)
            ));

            tex_filenames.push(tex_filename);
            packed_atlases.push(PackedAtlas {
                frames: atlas_frames,
                size: Size {
                    w: width,
                    h: height,
                },
                image: None,
                name: sheet_base(i),
                scale: 1.0,
            });
        }

        // Build export inputs for all sheets.
        let export_inputs: Vec<ExportInput<'_>> = packed_atlases
            .iter()
            .zip(tex_filenames.iter())
            .map(|(atlas, fname)| ExportInput {
                atlas,
                texture_filename: fname.clone(),
                pixel_format: pixel_format_str.to_string(),
            })
            .collect();

        // Try combined output first; fall back to per-sheet.
        if let Some(result) = exporter.combine(&export_inputs) {
            match result {
                Ok(content) => {
                    let data_filename = format!("{}.{}", name, exporter.file_extension());
                    let data_path = out_dir.join(&data_filename);
                    match std::fs::write(&data_path, content.as_bytes()) {
                        Ok(()) => self.state.log_info(t!(
                            "log.wrote_data",
                            path = data_path.display().to_string(),
                            bytes = content.len(),
                        )),
                        Err(e) => self
                            .state
                            .log_error(t!("log.write_data_failed", err = e.to_string())),
                    }
                }
                Err(e) => self
                    .state
                    .log_error(t!("log.export_failed", err = e.to_string())),
            }
        } else {
            for (i, input) in export_inputs.iter().enumerate() {
                match exporter.export(input) {
                    Ok(content) => {
                        let data_filename =
                            format!("{}.{}", sheet_base(i), exporter.file_extension());
                        let data_path = out_dir.join(&data_filename);
                        match std::fs::write(&data_path, content.as_bytes()) {
                            Ok(()) => self.state.log_info(t!(
                                "log.wrote_data",
                                path = data_path.display().to_string(),
                                bytes = content.len(),
                            )),
                            Err(e) => self
                                .state
                                .log_error(t!("log.write_data_failed", err = e.to_string())),
                        }
                    }
                    Err(e) => self.state.log_error(t!(
                        "log.export_failed_sheet",
                        i = i,
                        err = e.to_string()
                    )),
                }
            }
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        let mut new_sources: std::collections::BTreeSet<std::path::PathBuf> =
            std::collections::BTreeSet::new();

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
                            self.atlas_textures.clear();
                            self.state
                                .log_info(t!("log.opened", path = path.display().to_string()));
                        }
                        Err(e) => self
                            .state
                            .log_error(t!("log.parse_failed", err = e.to_string())),
                    },
                    Err(e) => self
                        .state
                        .log_error(t!("log.read_file_failed", err = e.to_string())),
                }
            } else if path.is_dir() {
                new_sources.insert(std::fs::canonicalize(&path).unwrap_or(path));
            } else if path.is_file() {
                if let Some(parent) = path.parent() {
                    new_sources.insert(
                        std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf()),
                    );
                }
            }
        }

        // If both /a and /a/b are in the drop, keep only /a — the walker covers children anyway.
        let all: Vec<_> = new_sources.iter().cloned().collect();
        for path in all
            .iter()
            .filter(|p| !all.iter().any(|other| other != *p && p.starts_with(other)))
        {
            self.state.add_source_path(path.clone());
        }
    }

    fn rebuild_watcher(&mut self) {
        self.file_watcher = None;
        self.watch_rx = None;

        if self.state.project.sources.is_empty() {
            return;
        }

        let (tx, rx) = mpsc::channel::<DebounceEventResult>();
        match new_debouncer(Duration::from_millis(500), tx) {
            Ok(mut debouncer) => {
                let watch_paths: Vec<_> = self
                    .state
                    .project
                    .sources
                    .iter()
                    .map(|s| {
                        if s.path.is_file() {
                            s.path.parent().unwrap_or(s.path.as_path()).to_path_buf()
                        } else {
                            s.path.clone()
                        }
                    })
                    .collect();
                let mut errors: Vec<String> = Vec::new();
                for path in &watch_paths {
                    if let Err(e) = debouncer.watcher().watch(path, RecursiveMode::Recursive) {
                        errors.push(format!("Could not watch {}: {e}", path.display()));
                    }
                }
                for err in errors {
                    self.state.log_warn(err);
                }
                self.file_watcher = Some(Box::new(debouncer));
                self.watch_rx = Some(rx);
            }
            Err(e) => self
                .state
                .log_warn(format!("Could not start file watcher: {e}")),
        }
    }

    fn poll_watcher(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.watch_rx else { return };
        let mut changed = false;
        loop {
            match rx.try_recv() {
                Ok(Ok(_)) => changed = true,
                Ok(Err(_)) | Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
        if changed && !self.state.packing {
            self.state.pending.pack = true;
            ctx.request_repaint();
        }
    }

    fn apply_ui_scale(&mut self, ctx: &egui::Context) {
        if self.native_pixels_per_point <= 0.0 {
            self.native_pixels_per_point = ctx.pixels_per_point();
        }
        let target = self.native_pixels_per_point * self.prefs.ui_scale;
        if (ctx.pixels_per_point() - target).abs() > 0.01 {
            ctx.set_pixels_per_point(target);
        }
    }
}
