//! Main application state and egui frame loop.

use crate::canvas::mouse::{CanvasMouseState, DragTarget};
use crate::engine::transform::{MatrixTransform, ReposAnchor};
use crate::engine::{parse_ass_into, Drawing};
use crate::i18n::t;
use crate::library::ShapeLibrary;
use crate::settings::AppSettings;
use crate::ui::{self, Tool};
use std::path::{Path, PathBuf};

pub struct AssDrawApp {
    pub drawing: Drawing,
    pub mouse: CanvasMouseState,
    pub tool: Tool,
    pub settings: AppSettings,
    pub library: ShapeLibrary,
    pub status: String,
    pub status_cmd: String,
    pub selected: Vec<u64>,
    pub last_cursor: (f64, f64),

    // Background Image
    pub bg_image: Option<egui::TextureHandle>,
    pub bg_path: Option<PathBuf>,
    pub bg_alpha: u8,
    pub bg_scale: f32,

    // Dialog flags
    pub show_transform: bool,
    pub show_settings: bool,
    pub show_about: bool,
    pub show_paste: bool,
    pub show_library: bool,
    pub show_manual: bool,
    pub show_bg_alpha_dialog: bool,

    // Transform parameters
    pub transform_scale: f32,
    pub transform_angle: f32,
    pub transform_selected_only: bool,
    pub active_matrix: MatrixTransform,
    pub transform_template_name: String,

    // Buffers
    pub paste_buffer: String,
    pub cmd_text: String,
    pub lib_name: String,
}

impl AssDrawApp {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        let settings = AppSettings::load();
        let bg_alpha = settings.bg_alpha_default;
        let mut a = Self {
            drawing: Drawing::new(),
            mouse: CanvasMouseState::default(),
            tool: Tool::Select,
            settings,
            library: ShapeLibrary::default(),
            status: "ASSDraw-rs hazır".into(),
            status_cmd: String::new(),
            selected: vec![],
            last_cursor: (0.0, 0.0),

            bg_image: None,
            bg_path: None,
            bg_alpha,
            bg_scale: 1.0,

            show_transform: false,
            show_settings: true,
            show_about: false,
            show_paste: false,
            show_library: true,
            show_manual: false,
            show_bg_alpha_dialog: false,

            transform_scale: 1.0,
            transform_angle: 0.0,
            transform_selected_only: false,
            active_matrix: MatrixTransform::default(),
            transform_template_name: "Özel (Custom)".into(),

            paste_buffer: String::new(),
            cmd_text: String::new(),
            lib_name: "Yeni Şekil".into(),
        };
        a.drawing.point_system.scale = 4.0;
        a.mouse.view_zoom = 2.0;
        let _ = parse_ass_into(&mut a.drawing, "m 8 -14 l 36 -25 l -39 -25");
        a.sync_cmd();
        a
    }

    pub fn set_status(&mut self, s: impl Into<String>) {
        self.status = s.into();
    }

    pub fn sync_cmd(&mut self) {
        self.cmd_text = self.drawing.to_ass_string_custom(self.settings.capitalize_cmds);
    }

    pub fn copy_ass(&mut self) {
        let line = if self.settings.use_multicolor {
            self.drawing.to_multicolor_ass(
                &self.settings.shape_fill,
                &self.settings.shape_outline,
                self.settings.border_width,
                self.settings.capitalize_cmds,
            )
        } else {
            self.drawing.to_full_ass_line(
                &self.settings.shape_fill,
                &self.settings.shape_outline,
                self.settings.border_width,
                self.settings.capitalize_cmds,
            )
        };
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(line);
            self.set_status(t(self.settings.language, "msg_copied"));
        }
    }

    pub fn clear(&mut self) {
        self.drawing.push_history_desc("Clear canvas");
        self.drawing.clear();
        self.mouse.clear_selection();
        self.mouse.temp_points.clear();
        self.selected.clear();
        self.sync_cmd();
        self.set_status(t(self.settings.language, "msg_cleared"));
    }

    pub fn delete_sel(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        self.drawing.push_history_desc("Delete selection");
        for id in self.selected.drain(..) {
            self.drawing.delete_point(id);
        }
        self.mouse.clear_selection();
        self.sync_cmd();
    }

    pub fn apply_paste(&mut self) {
        self.drawing.push_history_desc("Paste ASS");
        match parse_ass_into(&mut self.drawing, &self.paste_buffer) {
            Ok(n) => {
                self.sync_cmd();
                self.set_status(format!("{n} komut eklendi"));
            }
            Err(e) => self.set_status(format!("Ayrıştırma Hatası: {e}")),
        }
    }

    pub fn paste_from_clipboard(&mut self, ctx: &egui::Context) {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            // First check if clipboard has an image (tracing background image)
            if let Ok(img) = cb.get_image() {
                let size = [img.width, img.height];
                let ci = egui::ColorImage::from_rgba_unmultiplied(size, &img.bytes);
                self.bg_image = Some(ctx.load_texture("bg_image", ci, egui::TextureOptions::LINEAR));
                self.bg_path = None;
                self.mouse.drag_target = DragTarget::Background;
                self.set_status("Panodan arka plan resmi yapıştırıldı.");
                if self.settings.auto_ask_img_opacity {
                    self.show_bg_alpha_dialog = true;
                }
                return;
            }

            // Otherwise check if clipboard has text
            if let Ok(text) = cb.get_text() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    self.drawing.push_history_desc("Paste ASS");
                    match parse_ass_into(&mut self.drawing, trimmed) {
                        Ok(n) => {
                            self.sync_cmd();
                            self.set_status(format!("{n} komut yapıştırıldı."));
                        }
                        Err(e) => self.set_status(format!("Ayrıştırma Hatası: {e}")),
                    }
                }
            }
        }
    }

    pub fn apply_matrix(&mut self) {
        self.drawing.push_history_desc("Matrix transform");
        self.drawing.apply_matrix_transform(
            &self.active_matrix,
            self.transform_selected_only,
            &self.selected,
        );
        self.sync_cmd();
        self.set_status(t(self.settings.language, "msg_trans_applied"));
    }

    pub fn apply_transform(&mut self) {
        self.drawing.push_history_desc("Scale/Rotate");
        self.drawing
            .apply_scale_rotate(self.transform_scale as f64, self.transform_angle as f64);
        self.transform_scale = 1.0;
        self.transform_angle = 0.0;
        self.sync_cmd();
        self.set_status(t(self.settings.language, "msg_trans_applied"));
    }

    pub fn load_bg_from_path(&mut self, path: &Path, ctx: &egui::Context) {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let ci = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            self.bg_image = Some(ctx.load_texture("bg_image", ci, egui::TextureOptions::LINEAR));
            self.bg_path = Some(path.to_path_buf());
            self.set_status(t(self.settings.language, "msg_bg_loaded"));
            if self.settings.auto_ask_img_opacity {
                self.show_bg_alpha_dialog = true;
            }
        }
    }

    pub fn load_bg_from_bytes(&mut self, bytes: &[u8], ctx: &egui::Context) {
        if let Ok(img) = image::load_from_memory(bytes) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let ci = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            self.bg_image = Some(ctx.load_texture("bg_image", ci, egui::TextureOptions::LINEAR));
            self.set_status(t(self.settings.language, "msg_bg_loaded"));
            if self.settings.auto_ask_img_opacity {
                self.show_bg_alpha_dialog = true;
            }
        }
    }

    pub fn load_bg(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Görseller (Images)", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
        {
            self.load_bg_from_path(&path, ctx);
        }
    }

    pub fn save_json(&mut self) {
        if let Ok(j) = self.drawing.to_json() {
            if let Some(p) = rfd::FileDialog::new()
                .set_file_name("sekil.json")
                .add_filter("ASSDraw JSON", &["json"])
                .save_file()
            {
                let _ = std::fs::write(p, j);
                self.set_status("JSON dosyası kaydedildi");
            }
        }
    }

    pub fn load_json(&mut self) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("ASSDraw JSON", &["json"])
            .pick_file()
        {
            if let Ok(s) = std::fs::read_to_string(p) {
                if let Ok(d) = Drawing::from_json(&s) {
                    self.drawing.push_history_desc("Load JSON");
                    self.drawing = d;
                    self.sync_cmd();
                    self.set_status("JSON dosyası yüklendi");
                }
            }
        }
    }
}

impl eframe::App for AssDrawApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.settings.theme_dark {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        ctx.input(|i| {
            let c = i.modifiers.command || i.modifiers.ctrl;
            let shift = i.modifiers.shift;

            // Undo / Redo
            if c && i.key_pressed(egui::Key::Z) {
                if shift {
                    self.drawing.redo();
                } else {
                    self.drawing.undo();
                }
                self.sync_cmd();
            }
            if c && i.key_pressed(egui::Key::Y) {
                self.drawing.redo();
                self.sync_cmd();
            }

            // Clipboard
            if c && i.key_pressed(egui::Key::C) {
                self.copy_ass();
            }
            if c && i.key_pressed(egui::Key::V) {
                self.paste_from_clipboard(ctx);
            }
            if c && i.key_pressed(egui::Key::A) {
                self.mouse.select_all(&self.drawing);
            }
            if c && i.key_pressed(egui::Key::N) {
                self.clear();
            }
            if c && i.key_pressed(egui::Key::P) {
                self.mouse.preview_mode = !self.mouse.preview_mode;
            }

            // Reposition Shape Shortcuts: Ctrl+5, Ctrl+7, Ctrl+9, Ctrl+1, Ctrl+3
            if c && !shift {
                if i.key_pressed(egui::Key::Num5) {
                    self.drawing.push_history_desc("Center shape");
                    self.drawing.reposition_shape(ReposAnchor::Center);
                    self.sync_cmd();
                } else if i.key_pressed(egui::Key::Num7) {
                    self.drawing.push_history_desc("Top-Left shape");
                    self.drawing.reposition_shape(ReposAnchor::TopLeft);
                    self.sync_cmd();
                } else if i.key_pressed(egui::Key::Num9) {
                    self.drawing.push_history_desc("Top-Right shape");
                    self.drawing.reposition_shape(ReposAnchor::TopRight);
                    self.sync_cmd();
                } else if i.key_pressed(egui::Key::Num1) {
                    self.drawing.push_history_desc("Bottom-Left shape");
                    self.drawing.reposition_shape(ReposAnchor::BottomLeft);
                    self.sync_cmd();
                } else if i.key_pressed(egui::Key::Num3) {
                    self.drawing.push_history_desc("Bottom-Right shape");
                    self.drawing.reposition_shape(ReposAnchor::BottomRight);
                    self.sync_cmd();
                }
            }

            // Reposition Background Shortcut: Ctrl+Shift+5
            if c && shift && i.key_pressed(egui::Key::Num5) {
                self.mouse.bg_pan = egui::Vec2::ZERO;
            }

            // Delete
            if i.key_pressed(egui::Key::Delete) {
                if shift && self.bg_image.is_some() {
                    self.bg_image = None;
                    self.bg_path = None;
                    self.set_status("Arka plan kaldırıldı");
                } else {
                    self.delete_sel();
                }
            }

            // Function Keys & Tools
            if !c {
                if shift {
                    if i.key_pressed(egui::Key::F1) {
                        self.mouse.drag_target = DragTarget::Drawing;
                    }
                    if i.key_pressed(egui::Key::F2) {
                        self.mouse.drag_target = DragTarget::Background;
                    }
                    if i.key_pressed(egui::Key::F3) {
                        self.mouse.drag_target = DragTarget::Both;
                    }
                } else {
                    if i.key_pressed(egui::Key::F1) || i.key_pressed(egui::Key::V) {
                        self.tool = Tool::Select;
                    }
                    if i.key_pressed(egui::Key::F2) || i.key_pressed(egui::Key::M) {
                        self.tool = Tool::MoveTo;
                    }
                    if i.key_pressed(egui::Key::N) {
                        self.tool = Tool::MoveNoClose;
                    }
                    if i.key_pressed(egui::Key::F3) || i.key_pressed(egui::Key::L) {
                        self.tool = Tool::LineTo;
                    }
                    if i.key_pressed(egui::Key::F4) || i.key_pressed(egui::Key::B) {
                        self.tool = Tool::Bezier;
                    }
                    if i.key_pressed(egui::Key::F5) || i.key_pressed(egui::Key::S) {
                        self.tool = Tool::Spline;
                    }
                    if i.key_pressed(egui::Key::P) {
                        self.tool = Tool::ExtendSpline;
                    }
                    if i.key_pressed(egui::Key::C) {
                        self.tool = Tool::Close;
                    }
                    if i.key_pressed(egui::Key::F6) || i.key_pressed(egui::Key::R) {
                        self.tool = Tool::ScaleRotate;
                        self.show_transform = true;
                    }
                    if i.key_pressed(egui::Key::F7) {
                        self.tool = Tool::Bilinear;
                    }
                    if i.key_pressed(egui::Key::F8) {
                        self.tool = Tool::ColorPick;
                    }
                    if i.key_pressed(egui::Key::H) {
                        self.tool = Tool::Pan;
                    }
                }
            }
        });

        ui::top_bar(self, ctx);
        ui::cmd_panel(self, ctx);
        ui::right_dock_panel(self, ctx);
        ui::canvas(self, ctx);
        ui::dialogs(self, ctx);
    }
}
