//! Main application state and egui frame loop.

use crate::canvas::mouse::{CanvasMouseState, DragTarget};
use crate::engine::parse::{parse_ass_into, parse_styled_into_opts, parse_styled_replace};
use crate::engine::transform::{MatrixTransform, ReposAnchor};
use crate::engine::{parse_ass_into, Drawing, ShapeStyle};
use crate::i18n::t;
use crate::library::ShapeLibrary;
use crate::settings::AppSettings;
use crate::ui::{self, Tool};
use std::path::{Path, PathBuf};

/// Largest background image side we upload as a texture (bigger images are scaled down
/// and drawn scaled back up, so they keep their logical size).
const MAX_BG_SIDE: u32 = 4096;

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
    pub show_contour_color: bool,
    pub contour_color_idx: usize,
    pub contour_color_edit: crate::engine::Color,
    pub show_bg_alpha_dialog: bool,
    pub show_splash: bool,
    pub show_ribbon: bool,
    pub show_cmd_bar: bool,
    /// Library entry waiting for delete confirmation.
    pub pending_lib_delete: Option<usize>,

    // Transform parameters
    pub transform_scale: f32,
    pub transform_angle: f32,
    pub transform_selected_only: bool,
    pub active_matrix: MatrixTransform,
    pub transform_template_name: String,

    // Buffers
    pub paste_buffer: String,
    pub cmd_text: String,
    pub cmd_editing: bool,
    pub lib_name: String,
    /// Screen rectangle of the canvas (updated every frame by the canvas panel).
    pub canvas_rect: egui::Rect,
    /// Settings as last applied/saved (target of the Revert button).
    pub settings_saved: AppSettings,

    window_fitted: bool,
    /// Draw tool that was active before Shift temporarily switched to the select tool.
    tool_before_shift: Option<Tool>,
    confirm_quit_open: bool,
}

/// Keyboard/clipboard actions collected from the input state.
///
/// They are gathered inside `ctx.input(..)` and executed *afterwards*: running them inside the
/// closure would call `ctx.load_texture(..)` while the input lock is held, which deadlocks egui.
#[derive(Default)]
struct Actions {
    undo: bool,
    redo: bool,
    copy: bool,
    paste: bool,
    select_all: bool,
    clear: bool,
    toggle_preview: bool,
    /// (anchor, to_background): move the [0,0] origin of the view to a canvas / image corner.
    repos: Option<(ReposAnchor, bool)>,
    delete: bool,
    delete_bg: bool,
    tool: Option<Tool>,
    drag_target: Option<DragTarget>,
    pan: egui::Vec2,
    zoom: Option<f32>,
}

fn rgba_to_color_image(mut img: image::RgbaImage) -> (egui::ColorImage, f32) {
    let (w, h) = img.dimensions();
    let longest = w.max(h);
    let mut scale = 1.0_f32;
    if longest > MAX_BG_SIDE {
        let k = MAX_BG_SIDE as f32 / longest as f32;
        let nw = ((w as f32 * k).round() as u32).max(1);
        let nh = ((h as f32 * k).round() as u32).max(1);
        img = image::imageops::resize(&img, nw, nh, image::imageops::FilterType::Triangle);
        scale = longest as f32 / MAX_BG_SIDE as f32;
    }
    let size = [img.width() as usize, img.height() as usize];
    (egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), scale)
}

impl AssDrawApp {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        let settings = AppSettings::load();
        let lang = settings.language;
        let bg_alpha = settings.bg_alpha_default;
        let settings_saved = settings.clone();
        let mut a = Self {
            drawing: Drawing::new(),
            mouse: CanvasMouseState::default(),
            tool: Tool::Select,
            settings,
            library: ShapeLibrary::load(),
            status: t(lang, "msg_ready").into(),
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
            show_contour_color: false,
            contour_color_idx: 0,
            contour_color_edit: crate::engine::Color::from_rgb(0, 102, 255),
            show_bg_alpha_dialog: false,
            show_splash: false,
            show_ribbon: true,
            show_cmd_bar: true,
            pending_lib_delete: None,

            transform_scale: 1.0,
            transform_angle: 0.0,
            transform_selected_only: false,
            active_matrix: MatrixTransform::default(),
            transform_template_name: t(lang, "tmpl_custom").into(),

            paste_buffer: String::new(),
            cmd_text: String::new(),
            cmd_editing: false,
            lib_name: String::new(),
            canvas_rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0)),
            settings_saved,

            window_fitted: false,
            tool_before_shift: None,
            confirm_quit_open: false,
        };
        a.show_splash = !a.settings.no_splash;
        // Like the original: a fresh canvas holds a single `m 0 0`.
        let _ = parse_ass_into(&mut a.drawing, "m 0 0");
        a.sync_cmd();
        a
    }

    /// Move the view's [0,0] origin to a corner / the centre of the canvas, or of the background
    /// image (the original's "Reposition [0,0]" items). The drawing itself is not touched.
    pub fn reposition_origin(&mut self, anchor: ReposAnchor, to_bg: bool) {
        let origin = self.mouse.origin(self.canvas_rect);
        let area = if to_bg {
            let Some(tex) = &self.bg_image else { return };
            let size = tex.size_vec2() * self.bg_scale * self.mouse.bg_zoom;
            egui::Rect::from_center_size(origin + self.mouse.bg_pan, size)
        } else {
            self.canvas_rect
        };
        let target = match anchor {
            ReposAnchor::Center => area.center(),
            ReposAnchor::TopLeft => area.left_top(),
            ReposAnchor::TopRight => area.right_top(),
            ReposAnchor::BottomLeft => area.left_bottom(),
            ReposAnchor::BottomRight => area.right_bottom(),
        };
        let shift = target - origin;
        self.mouse.pan += shift;
        self.mouse.bg_pan -= shift;
    }

    /// Open the colour dialog for the sub-shape whose outer contour has index `idx`.
    pub fn open_color_dialog(&mut self, idx: usize) {
        self.contour_color_idx = idx;
        self.contour_color_edit = self
            .drawing
            .get_contour_color(idx)
            .copied()
            .unwrap_or(self.settings.shape_fill);
        self.show_contour_color = true;
    }

    pub fn set_status(&mut self, s: impl Into<String>) {
        self.status = s.into();
    }

    /// The ASS line shown in (and copied from) the command box.
    pub fn ass_line(&self) -> String {
        self.drawing.to_styled_ass(
            &self.settings.shape_style(),
            self.settings.capitalize_cmds,
            self.settings.use_multicolor,
        )
    }

    /// Append `text` (ASS drawing, optionally with colour/alpha/border tags) to the drawing and
    /// adopt the style found in it. Returns the number of commands added.
    /// On parse error the drawing is left unchanged.
    fn import_ass(&mut self, text: &str) -> Result<usize, crate::engine::ParseError> {
        let mut style: ShapeStyle = self.settings.shape_style();
        let mut scratch = self.drawing.clone_geometry();
        let n = parse_styled_into_opts(&mut scratch, text, &mut style, self.settings.parse_spc)?;
        // Success only — snapshot current, then commit.
        self.drawing.push_history_desc("Paste ASS");
        self.drawing.copy_geometry_from(&scratch);
        self.settings.apply_shape_style(&style);
        Ok(n)
    }

    pub fn sync_cmd(&mut self) {
        self.cmd_text = self.ass_line();
    }

    pub fn copy_ass(&mut self) {
        let lang = self.settings.language;
        let line = self.ass_line();
        match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(line)) {
            Ok(()) => self.set_status(t(lang, "msg_copied")),
            Err(_) => self.set_status(t(lang, "msg_clip_error")),
        }
    }

    pub fn clear(&mut self) {
        self.drawing.push_history_desc("Clear canvas");
        self.drawing.clear_shape();
        self.mouse.clear_selection();
        self.mouse.temp_points.clear();
        self.selected.clear();
        self.sync_cmd();
        self.set_status(t(self.settings.language, "msg_cleared"));
    }

    pub fn delete_sel(&mut self) {
        // Prefer live canvas selection (keyboard Delete runs before the canvas syncs `self.selected`).
        let ids: Vec<_> = if !self.mouse.selected.is_empty() {
            self.mouse.selected.clone()
        } else {
            self.selected.clone()
        };
        if ids.is_empty() {
            return;
        }
        self.drawing.push_history_desc("Delete selection");
        for id in ids {
            self.drawing.delete_point(id);
        }
        self.mouse.clear_selection();
        self.selected.clear();
        self.sync_cmd();
    }

    /// Append the text of the paste dialog to the drawing.
    pub fn apply_paste(&mut self) {
        let lang = self.settings.language;
        let text = self.paste_buffer.clone();
        match self.import_ass(&text) {
            Ok(n) => {
                self.sync_cmd();
                self.set_status(format!("{n} {}", t(lang, "msg_cmds_added")));
            }
            Err(e) => self.set_status(format!("{}: {e}", t(lang, "msg_parse_error"))),
        }
    }

    /// Replace the drawing with the text typed into the command box (Enter). Parsing is tolerant
    /// (like the original), so this never fails; unknown words are skipped.
    pub fn apply_cmd_text(&mut self) {
        let lang = self.settings.language;
        let text = self.cmd_text.clone();
        let mut style = self.settings.shape_style();
        match parse_styled_replace(&mut self.drawing, &text, &mut style, self.settings.parse_spc) {
            Ok(n) => {
                self.settings.apply_shape_style(&style);
                self.mouse.clear_selection();
                self.mouse.reset_transform();
                self.selected.clear();
                self.sync_cmd();
                self.set_status(format!("{n} {}", t(lang, "msg_cmds_added")));
            }
            Err(e) => self.set_status(format!("{}: {e}", t(lang, "msg_parse_error"))),
        }
    }

    /// Paste from the system clipboard: an image becomes the tracing background,
    /// text is parsed as ASS drawing commands.
    pub fn paste_from_clipboard(&mut self, ctx: &egui::Context) {
        let lang = self.settings.language;
        let mut cb = match arboard::Clipboard::new() {
            Ok(cb) => cb,
            Err(_) => {
                self.set_status(t(lang, "msg_clip_error"));
                return;
            }
        };

        if let Ok(img) = cb.get_image() {
            let rgba = image::RgbaImage::from_raw(
                img.width as u32,
                img.height as u32,
                img.bytes.into_owned(),
            );
            if let Some(rgba) = rgba {
                self.set_background(ctx, rgba);
                self.bg_path = None;
                self.mouse.drag_target = DragTarget::Background;
                self.set_status(t(lang, "msg_bg_pasted"));
                return;
            }
        }

        if let Ok(text) = cb.get_text() {
            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() {
                // Same result as pasting into the command box and pressing Enter.
                self.cmd_text = trimmed;
                self.apply_cmd_text();
                return;
            }
        }

        self.set_status(t(lang, "msg_clip_empty"));
    }

    pub fn apply_matrix(&mut self) {
        let sel = if !self.mouse.selected.is_empty() {
            self.mouse.selected.clone()
        } else {
            self.selected.clone()
        };
        self.drawing.push_history_desc("Matrix transform");
        self.drawing.apply_matrix_transform(
            &self.active_matrix,
            self.transform_selected_only,
            &sel,
        );
        self.mouse.reset_transform();
        self.sync_cmd();
        self.set_status(t(self.settings.language, "msg_trans_applied"));
    }

    pub fn apply_transform(&mut self) {
        self.drawing.push_history_desc("Scale/Rotate");
        self.drawing
            .apply_scale_rotate(self.transform_scale as f64, self.transform_angle as f64);
        self.transform_scale = 1.0;
        self.transform_angle = 0.0;
        self.mouse.reset_transform();
        self.sync_cmd();
        self.set_status(t(self.settings.language, "msg_trans_applied"));
    }

    fn set_background(&mut self, ctx: &egui::Context, rgba: image::RgbaImage) {
        let (ci, scale) = rgba_to_color_image(rgba);
        self.bg_image = Some(ctx.load_texture("bg_image", ci, egui::TextureOptions::LINEAR));
        self.bg_scale = scale;
        self.bg_alpha = self.settings.bg_alpha_default;
        if self.settings.auto_ask_img_opacity {
            self.show_bg_alpha_dialog = true;
        }
    }

    pub fn load_bg_from_path(&mut self, path: &Path, ctx: &egui::Context) {
        let lang = self.settings.language;
        match image::open(path) {
            Ok(img) => {
                self.set_background(ctx, img.to_rgba8());
                self.bg_path = Some(path.to_path_buf());
                self.set_status(t(lang, "msg_bg_loaded"));
            }
            Err(_) => self.set_status(t(lang, "msg_bg_failed")),
        }
    }

    pub fn load_bg_from_bytes(&mut self, bytes: &[u8], ctx: &egui::Context) {
        let lang = self.settings.language;
        match image::load_from_memory(bytes) {
            Ok(img) => {
                self.set_background(ctx, img.to_rgba8());
                self.set_status(t(lang, "msg_bg_loaded"));
            }
            Err(_) => self.set_status(t(lang, "msg_bg_failed")),
        }
    }

    pub fn load_bg(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
        {
            self.load_bg_from_path(&path, ctx);
        }
    }

    pub fn save_json(&mut self) {
        if let Ok(j) = self.drawing.to_json() {
            if let Some(p) = rfd::FileDialog::new()
                .set_file_name("shape.json")
                .add_filter("ASSDraw JSON", &["json"])
                .save_file()
            {
                let _ = std::fs::write(p, j);
                self.set_status(t(self.settings.language, "msg_json_saved"));
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
                    self.set_status(t(self.settings.language, "msg_json_loaded"));
                }
            }
        }
    }

    /// Shrink the window once if the saved/default size does not fit the monitor
    /// (otherwise the canvas hangs off-screen and has to be resized by hand).
    fn fit_window_to_monitor(&mut self, ctx: &egui::Context) {
        if self.window_fitted {
            return;
        }
        let (monitor, inner) = ctx.input(|i| {
            let vp = i.viewport();
            (vp.monitor_size, vp.inner_rect.map(|r| r.size()))
        });
        let Some(monitor) = monitor else { return };
        self.window_fitted = true;
        if let Some(cur) = inner {
            let max = egui::vec2(monitor.x * 0.92, monitor.y * 0.85);
            if cur.x > max.x || cur.y > max.y {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    cur.x.min(max.x),
                    cur.y.min(max.y),
                )));
            }
        }
    }

    /// Welcome screen shown at startup (can be disabled for good).
    fn show_splash_window(&mut self, ctx: &egui::Context) {
        if !self.show_splash {
            return;
        }
        let lang = self.settings.language;
        let mut start = false;
        let mut dont_show = self.settings.no_splash;
        egui::Window::new(t(lang, "splash_welcome"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Image::new(egui::include_image!("../../assets/welcome-large.bmp"))
                            .max_height(300.0),
                    );
                    ui.add_space(6.0);
                    ui.checkbox(&mut dont_show, t(lang, "splash_dont_show"));
                    if ui.button(t(lang, "btn_ok_start")).clicked() {
                        start = true;
                    }
                });
            });
        if dont_show != self.settings.no_splash {
            self.settings.no_splash = dont_show;
            self.settings.save();
        }
        if start {
            self.show_splash = false;
        }
    }

    fn show_library_delete_confirm(&mut self, ctx: &egui::Context) {
        let Some(idx) = self.pending_lib_delete else { return };
        let lang = self.settings.language;
        let mut delete = false;
        let mut cancel = false;
        egui::Window::new(t(lang, "lib_ctx_delete"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(t(lang, "lib_delete_q"));
                ui.horizontal(|ui| {
                    if ui.button(t(lang, "lib_ctx_delete")).clicked() {
                        delete = true;
                    }
                    if ui.button(t(lang, "dlg_cancel")).clicked() {
                        cancel = true;
                    }
                });
            });
        if delete {
            self.library.remove(idx);
            self.library.save();
            self.set_status(t(lang, "msg_shapes_deleted"));
            self.pending_lib_delete = None;
        } else if cancel {
            self.pending_lib_delete = None;
        }
    }

    /// Holding Shift alone temporarily switches a draw tool (M/L/B) to the drag tool,
    /// releasing it restores the tool (ASSDraw3 behaviour).
    fn update_shift_drag_mode(&mut self, ctx: &egui::Context) {
        let shift_only = ctx.input(|i| i.modifiers.shift && !i.modifiers.command && !i.modifiers.alt);
        if shift_only {
            if self.tool_before_shift.is_none()
                && matches!(self.tool, Tool::MoveTo | Tool::LineTo | Tool::Bezier)
            {
                self.tool_before_shift = Some(self.tool);
                self.tool = Tool::Select;
            }
        } else if let Some(prev) = self.tool_before_shift.take() {
            if self.tool == Tool::Select {
                self.tool = prev;
            }
        }
    }

    /// Ask before closing when "confirm quit" is enabled and there is something drawn.
    fn handle_close_request(&mut self, ctx: &egui::Context) {
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested
            && self.settings.confirm_quit
            && !self.drawing.commands.is_empty()
            && !self.confirm_quit_open
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_quit_open = true;
        }
    }

    pub fn show_confirm_quit(&mut self, ctx: &egui::Context) {
        if !self.confirm_quit_open {
            return;
        }
        let lang = self.settings.language;
        let mut quit = false;
        let mut stay = false;
        egui::Window::new(t(lang, "quit_title"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(t(lang, "quit_question"));
                ui.horizontal(|ui| {
                    if ui.button(t(lang, "quit_yes")).clicked() {
                        quit = true;
                    }
                    if ui.button(t(lang, "quit_no")).clicked() {
                        stay = true;
                    }
                });
            });
        if quit {
            self.settings.confirm_quit = false; // let the next close request through
            self.confirm_quit_open = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if stay {
            self.confirm_quit_open = false;
        }
    }

    fn collect_actions(ctx: &egui::Context) -> Actions {
        ctx.input(|i| {
            let mut a = Actions::default();
            let c = i.modifiers.command;
            let shift = i.modifiers.shift;

            // egui-winit turns Ctrl+C / Ctrl+V into `Event::Copy` / `Event::Paste` and does NOT
            // forward the key press, so `key_pressed(C/V)` alone never fires. Listen for both.
            for ev in &i.events {
                match ev {
                    egui::Event::Copy => a.copy = true,
                    egui::Event::Paste(_) => a.paste = true,
                    _ => {}
                }
            }
            if c && i.key_pressed(egui::Key::C) {
                a.copy = true;
            }
            // Ctrl+V is swallowed when the clipboard holds an image only, so Ctrl+I is the
            // reliable "paste image" shortcut (the menu entries work as well).
            if c && (i.key_pressed(egui::Key::V) || i.key_pressed(egui::Key::I)) {
                a.paste = true;
            }

            if c && i.key_pressed(egui::Key::Z) {
                if shift {
                    a.redo = true;
                } else {
                    a.undo = true;
                }
            }
            if c && i.key_pressed(egui::Key::Y) {
                a.redo = true;
            }
            if c && i.key_pressed(egui::Key::A) {
                a.select_all = true;
            }
            if c && i.key_pressed(egui::Key::N) {
                a.clear = true;
            }
            if c && i.key_pressed(egui::Key::P) {
                a.toggle_preview = true;
            }

            if c {
                let anchors = [
                    (egui::Key::Num5, ReposAnchor::Center),
                    (egui::Key::Num7, ReposAnchor::TopLeft),
                    (egui::Key::Num9, ReposAnchor::TopRight),
                    (egui::Key::Num1, ReposAnchor::BottomLeft),
                    (egui::Key::Num3, ReposAnchor::BottomRight),
                ];
                for (key, anchor) in anchors {
                    if i.key_pressed(key) {
                        a.repos = Some((anchor, shift));
                        break;
                    }
                }
            }

            // Arrow keys move the canvas (Ctrl = 10x), PageUp/PageDown zoom (as in ASSDraw3).
            if !shift {
                let step = if c { 10.0 } else { 1.0 };
                let mut d = egui::Vec2::ZERO;
                if i.key_pressed(egui::Key::ArrowUp) {
                    d.y -= step;
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    d.y += step;
                }
                if i.key_pressed(egui::Key::ArrowLeft) {
                    d.x -= step;
                }
                if i.key_pressed(egui::Key::ArrowRight) {
                    d.x += step;
                }
                a.pan = d;
                let amount = if c { 0.1 } else { 1.0 };
                if i.key_pressed(egui::Key::PageUp) {
                    a.zoom = Some(amount);
                }
                if i.key_pressed(egui::Key::PageDown) {
                    a.zoom = Some(-amount);
                }
            }

            if i.key_pressed(egui::Key::Delete) {
                if shift {
                    a.delete_bg = true;
                } else {
                    a.delete = true;
                }
            }

            if !c {
                if shift {
                    if i.key_pressed(egui::Key::F1) {
                        a.drag_target = Some(DragTarget::Drawing);
                    }
                    if i.key_pressed(egui::Key::F2) {
                        a.drag_target = Some(DragTarget::Background);
                    }
                    if i.key_pressed(egui::Key::F3) {
                        a.drag_target = Some(DragTarget::Both);
                    }
                } else {
                    let tools = [
                        (egui::Key::F1, Tool::Select),
                        (egui::Key::V, Tool::Select),
                        (egui::Key::F2, Tool::MoveTo),
                        (egui::Key::M, Tool::MoveTo),
                        (egui::Key::N, Tool::MoveNoClose),
                        (egui::Key::F3, Tool::LineTo),
                        (egui::Key::L, Tool::LineTo),
                        (egui::Key::F4, Tool::Bezier),
                        (egui::Key::B, Tool::Bezier),
                        (egui::Key::F5, Tool::Delete),
                        (egui::Key::S, Tool::Spline),
                        (egui::Key::P, Tool::ExtendSpline),
                        (egui::Key::C, Tool::Close),
                        (egui::Key::F6, Tool::ScaleRotate),
                        (egui::Key::R, Tool::ScaleRotate),
                        (egui::Key::F7, Tool::Bilinear),
                        (egui::Key::F8, Tool::ColorPick),
                        (egui::Key::H, Tool::Pan),
                    ];
                    for (key, tool) in tools {
                        if i.key_pressed(key) {
                            a.tool = Some(tool);
                        }
                    }
                }
            }
            a
        })
    }

    fn run_actions(&mut self, ctx: &egui::Context, a: Actions) {
        if a.undo {
            let desc = self
                .drawing
                .top_undo_desc()
                .unwrap_or("Edit")
                .to_string();
            self.drawing.undo();
            self.mouse.clear_selection();
            self.selected.clear();
            self.mouse.reset_transform();
            self.sync_cmd();
            self.set_status(format!("{}: {desc}", t(self.settings.language, "action_undo")));
        }
        if a.redo {
            let desc = self
                .drawing
                .top_redo_desc()
                .unwrap_or("Edit")
                .to_string();
            self.drawing.redo();
            self.mouse.clear_selection();
            self.selected.clear();
            self.mouse.reset_transform();
            self.sync_cmd();
            self.set_status(format!("{}: {desc}", t(self.settings.language, "action_redo")));
        }
        if a.copy {
            self.copy_ass();
        }
        if a.paste {
            self.paste_from_clipboard(ctx);
        }
        if a.select_all {
            self.mouse.select_all(&self.drawing);
            self.selected = self.mouse.selected.clone();
        }
        if a.clear {
            self.clear();
        }
        if a.toggle_preview {
            self.mouse.preview_mode = !self.mouse.preview_mode;
        }
        if let Some((anchor, to_bg)) = a.repos {
            self.reposition_origin(anchor, to_bg);
        }
        if a.delete_bg && self.bg_image.is_some() {
            self.bg_image = None;
            self.bg_path = None;
            self.set_status(t(self.settings.language, "msg_bg_removed"));
        }
        if a.delete {
            self.delete_sel();
        }
        if a.pan != egui::Vec2::ZERO {
            self.mouse.pan_by(a.pan);
        }
        if let Some(amount) = a.zoom {
            self.mouse.zoom_by(self.canvas_rect, amount, None);
        }
        if let Some(dt) = a.drag_target {
            self.mouse.drag_target = dt;
        }
        if let Some(tool) = a.tool {
            self.tool = tool;
        }
    }
}

impl eframe::App for AssDrawApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.settings.save();
        self.library.save();
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let preference = if self.settings.theme_system {
            egui::ThemePreference::System
        } else if self.settings.theme_dark {
            egui::ThemePreference::Dark
        } else {
            egui::ThemePreference::Light
        };
        ctx.set_theme(preference);

        self.fit_window_to_monitor(ctx);

        // While a text field has focus, typed letters/Ctrl+V belong to that field.
        if !ctx.wants_keyboard_input() {
            let actions = Self::collect_actions(ctx);
            self.run_actions(ctx, actions);
            self.update_shift_drag_mode(ctx);
        }
        self.handle_close_request(ctx);

        ui::top_bar(self, ctx);
        ui::cmd_panel(self, ctx);
        ui::right_dock_panel(self, ctx);
        ui::canvas(self, ctx);
        ui::dialogs(self, ctx);
        self.show_confirm_quit(ctx);
        self.show_splash_window(ctx);
        self.show_library_delete_confirm(ctx);
    }
}
