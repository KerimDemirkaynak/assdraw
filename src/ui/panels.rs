//! Application Panels: Top Menu Bar, Tool Ribbon, Bottom Command Editor, and Right Dock Panel.
//! Faithfully mirrors original ASSDraw3 layout (Image 3) with SVG icons.

use crate::app::AssDrawApp;
use crate::canvas::mouse::DragTarget;
use crate::engine::transform::ReposAnchor;
use crate::engine::{Color, Drawing};
use crate::i18n::{t, Language};
use crate::ui::icons::{icon, themed, Icon};
use crate::ui::Tool;
use egui::{Color32, Pos2, RichText};

#[derive(Clone, Copy)]
enum LibAction {
    Load,
    CopyAss,
    Overwrite,
    Delete,
}

pub fn top_bar(app: &mut AssDrawApp, ctx: &egui::Context) {
    let lang = app.settings.language;

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // 1. Canvas
            ui.menu_button(t(lang, "menu_canvas"), |ui| {
                if ui.button(format!("{} (Ctrl+N)", t(lang, "action_clear"))).clicked() {
                    app.clear();
                    ui.close_menu();
                }
                if ui
                    .checkbox(
                        &mut app.mouse.preview_mode,
                        format!("{} (Ctrl+P)", t(lang, "action_preview")),
                    )
                    .clicked()
                {
                    ui.close_menu();
                }
                if ui.button(t(lang, "menu_transform")).clicked() {
                    app.show_transform = true;
                    ui.close_menu();
                }
                if ui.button(format!("{} (Ctrl+V)", t(lang, "menu_paste"))).clicked() {
                    app.paste_from_clipboard(ctx);
                    ui.close_menu();
                }
                ui.separator();
                let undo_lbl = if let Some(desc) = app.drawing.top_undo_desc() {
                    format!("{} ({}) Ctrl+Z", t(lang, "action_undo"), desc)
                } else {
                    format!("{} (Ctrl+Z)", t(lang, "action_undo"))
                };
                if ui.add_enabled(app.drawing.can_undo(), egui::Button::new(undo_lbl)).clicked() {
                    app.drawing.undo();
                    app.sync_cmd();
                    ui.close_menu();
                }
                let redo_lbl = if let Some(desc) = app.drawing.top_redo_desc() {
                    format!("{} ({}) Ctrl+Y", t(lang, "action_redo"), desc)
                } else {
                    format!("{} (Ctrl+Y)", t(lang, "action_redo"))
                };
                if ui.add_enabled(app.drawing.can_redo(), egui::Button::new(redo_lbl)).clicked() {
                    app.drawing.redo();
                    app.sync_cmd();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t(lang, "menu_open_json")).clicked() {
                    app.load_json();
                    ui.close_menu();
                }
                if ui.button(t(lang, "menu_save_json")).clicked() {
                    app.save_json();
                    ui.close_menu();
                }
            });

            // 2. Kip (Mode)
            ui.menu_button(t(lang, "menu_mode"), |ui| {
                if ui.radio(app.tool == Tool::Select, format!("{}   F1", t(lang, "tool_select"))).clicked() {
                    app.tool = Tool::Select;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::MoveTo, format!("{}   F2", t(lang, "tool_m"))).clicked() {
                    app.tool = Tool::MoveTo;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::LineTo, format!("{}   F3", t(lang, "tool_l"))).clicked() {
                    app.tool = Tool::LineTo;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::Bezier, format!("{}   F4", t(lang, "tool_b"))).clicked() {
                    app.tool = Tool::Bezier;
                    ui.close_menu();
                }
                for (tool, key, shortcut) in [
                    (Tool::MoveNoClose, "tool_n", "N"),
                    (Tool::Spline, "tool_s", "S"),
                    (Tool::ExtendSpline, "tool_p", "P"),
                    (Tool::Close, "tool_c", "C"),
                ] {
                    if ui
                        .radio(app.tool == tool, format!("{}   {shortcut}", t(lang, key)))
                        .clicked()
                    {
                        app.tool = tool;
                        ui.close_menu();
                    }
                }
                if ui
                    .radio(app.tool == Tool::Delete, format!("{}   F5", t(lang, "tool_del")))
                    .clicked()
                {
                    app.tool = Tool::Delete;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::ScaleRotate, format!("{}   F6", t(lang, "tool_scale_rotate"))).clicked() {
                    app.tool = Tool::ScaleRotate;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::Bilinear, format!("{}   F7", t(lang, "tool_bilinear"))).clicked() {
                    app.tool = Tool::Bilinear;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::ColorPick, t(lang, "rb_color")).clicked() {
                    app.tool = Tool::ColorPick;
                    ui.close_menu();
                }
            });

            // 3. Arka plan (Background)
            ui.menu_button(t(lang, "menu_bg"), |ui| {
                if ui.radio(app.mouse.drag_target == DragTarget::Drawing, format!("{}   Shift+F1", t(lang, "bg_drag_drawing"))).clicked() {
                    app.mouse.drag_target = DragTarget::Drawing;
                    ui.close_menu();
                }
                if ui.radio(app.mouse.drag_target == DragTarget::Background, format!("{}   Shift+F2", t(lang, "bg_drag_bg"))).clicked() {
                    app.mouse.drag_target = DragTarget::Background;
                    ui.close_menu();
                }
                if ui.radio(app.mouse.drag_target == DragTarget::Both, format!("{}   Shift+F3", t(lang, "bg_drag_both"))).clicked() {
                    app.mouse.drag_target = DragTarget::Both;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(format!("{} (Ctrl+I)", t(lang, "bg_paste_image"))).clicked() {
                    app.paste_from_clipboard(ctx);
                    ui.close_menu();
                }
                if ui.button(t(lang, "bg_load_file")).clicked() {
                    app.load_bg(ctx);
                    ui.close_menu();
                }
                if ui.button(t(lang, "action_bg_alpha")).clicked() {
                    app.show_bg_alpha_dialog = true;
                    ui.close_menu();
                }
                if app.bg_image.is_some() && ui.button(format!("{}   Shift+Del", t(lang, "bg_remove"))).clicked() {
                    app.bg_image = None;
                    app.bg_path = None;
                    app.set_status(t(lang, "msg_bg_removed"));
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button(t(lang, "reposition_00"), |ui| {
                    for (anchor, key, hint) in REPOS_ITEMS {
                        if ui.button(format!("{}   Ctrl+Shift+{hint}", t(lang, key))).clicked() {
                            app.reposition_origin(anchor, true);
                            ui.close_menu();
                        }
                    }
                });
            });

            // 4. Çalışma alanı (Workspace)
            ui.menu_button(t(lang, "menu_workspace"), |ui| {
                if ui.checkbox(&mut app.show_library, t(lang, "dialog_library_title")).clicked() {
                    ui.close_menu();
                }
                if ui.checkbox(&mut app.show_settings, t(lang, "dialog_settings_title")).clicked() {
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button(t(lang, "menu_theme"), |ui| {
                    let system = app.settings.theme_system;
                    let dark = app.settings.theme_dark;
                    if ui.radio(!system && !dark, t(lang, "theme_light")).clicked() {
                        app.settings.theme_system = false;
                        app.settings.theme_dark = false;
                        ui.close_menu();
                    }
                    if ui.radio(!system && dark, t(lang, "theme_dark")).clicked() {
                        app.settings.theme_system = false;
                        app.settings.theme_dark = true;
                        ui.close_menu();
                    }
                    if ui.radio(system, t(lang, "theme_system")).clicked() {
                        app.settings.theme_system = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button(t(lang, "menu_toolbars"), |ui| {
                    ui.checkbox(&mut app.show_ribbon, t(lang, "tb_ribbon"));
                    ui.checkbox(&mut app.show_cmd_bar, t(lang, "tb_cmd"));
                });
                ui.separator();
                ui.menu_button(t(lang, "reposition_00"), |ui| {
                    for (anchor, key, hint) in REPOS_ITEMS {
                        if ui.button(format!("{}   Ctrl+{hint}", t(lang, key))).clicked() {
                            app.reposition_origin(anchor, false);
                            ui.close_menu();
                        }
                    }
                });
                ui.separator();
                if ui.button(t(lang, "reset_workspace")).clicked() {
                    app.mouse.pan = egui::Vec2::ZERO;
                    app.mouse.view_zoom = crate::canvas::mouse::DEFAULT_SCALE;
                    app.mouse.bg_pan = egui::Vec2::ZERO;
                    app.mouse.bg_zoom = 1.0;
                    ui.close_menu();
                }
            });

            // 5. Yardım (Help)
            ui.menu_button(t(lang, "menu_help"), |ui| {
                if ui.button(t(lang, "help_manual")).clicked() {
                    app.show_manual = true;
                    ui.close_menu();
                }
                if ui.button(t(lang, "help_about")).clicked() {
                    app.show_about = true;
                    ui.close_menu();
                }
            });

            // 6. Dil (Language)
            ui.menu_button(t(lang, "menu_lang"), |ui| {
                for (lang_code, lang_name) in [
                    (Language::En, "English"),
                    (Language::Tr, "Türkçe"),
                    (Language::Fr, "Français"),
                    (Language::De, "Deutsch"),
                    (Language::Es, "Español"),
                    (Language::ZhTw, "繁體中文"),
                    (Language::Vi, "Tiếng Việt"),
                ] {
                    if ui.radio(app.settings.language == lang_code, lang_name).clicked() {
                        app.settings.language = lang_code;
                        app.settings.save();
                        ui.close_menu();
                    }
                }
            });
        });
    });

    // Tool ribbon. `horizontal_wrapped` lets long translations flow onto a second row
    // instead of pushing the window wider than the screen.
    let show_ribbon = app.show_ribbon;
    egui::TopBottomPanel::top("tool_ribbon").show_animated(ctx, show_ribbon, |ui| {
        ui.horizontal_wrapped(|ui| {
            if ribbon_tool_btn(ui, Icon::Broom, t(lang, "rb_clear"), false)
                .on_hover_text(t(lang, "rb_clear_tip"))
                .clicked()
            {
                app.clear();
            }

            if ribbon_tool_btn(ui, Icon::Eye, t(lang, "rb_preview"), app.mouse.preview_mode)
                .on_hover_text(t(lang, "rb_preview_tip"))
                .clicked()
            {
                app.mouse.preview_mode = !app.mouse.preview_mode;
            }

            ui.add_sized(
                [90.0, 18.0],
                egui::Slider::new(&mut app.mouse.view_zoom, 1.0..=50.0).show_value(false),
            );

            ui.separator();

            let group1 = [
                (Tool::Select, t(lang, "rb_drag"), Icon::ArrowPointer),
                (Tool::MoveTo, t(lang, "rb_move"), Icon::LocationDot),
                (Tool::LineTo, t(lang, "rb_line"), Icon::Slash),
                (Tool::Bezier, t(lang, "rb_bezier"), Icon::BezierCurve),
                (Tool::Delete, t(lang, "rb_delete"), Icon::Eraser),
            ];
            for (tool, label, icon) in group1 {
                if ribbon_tool_btn(ui, icon, label, app.tool == tool)
                    .on_hover_text(tool.tooltip(lang))
                    .clicked()
                {
                    app.tool = tool;
                }
            }

            ui.separator();

            if ribbon_tool_btn(
                ui,
                Icon::ArrowsRotate,
                t(lang, "rb_scale_rot"),
                app.tool == Tool::ScaleRotate,
            )
            .clicked()
            {
                app.tool = Tool::ScaleRotate;
            }

            if ribbon_tool_btn(
                ui,
                Icon::DrawPolygon,
                t(lang, "rb_bilinear"),
                app.tool == Tool::Bilinear,
            )
            .clicked()
            {
                app.tool = Tool::Bilinear;
            }

            ui.separator();

            if ribbon_tool_btn(ui, Icon::Palette, t(lang, "rb_color"), app.tool == Tool::ColorPick)
                .on_hover_text(t(lang, "rb_color_tip"))
                .clicked()
            {
                app.tool = if app.tool == Tool::ColorPick {
                    Tool::Select
                } else {
                    Tool::ColorPick
                };
            }
        });
    });
}

/// One ribbon button: icon on top, label below. The button grows with the label so that
/// long translations never spill out of it (and is clipped as a last resort).
fn ribbon_tool_btn(ui: &mut egui::Ui, which: Icon, label: &str, is_active: bool) -> egui::Response {
    let font = egui::FontId::proportional(11.0);
    let measure = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), Color32::WHITE);
    let width = (measure.size().x + 14.0).clamp(52.0, 120.0);

    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 42.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let bg_color = if is_active {
            Color32::from_rgb(0, 114, 206)
        } else if response.hovered() {
            visuals.bg_fill
        } else {
            Color32::TRANSPARENT
        };
        let stroke = if is_active {
            egui::Stroke::new(1.0_f32, Color32::from_rgb(140, 190, 255))
        } else if response.hovered() {
            visuals.bg_stroke
        } else {
            egui::Stroke::NONE
        };
        let fg = if is_active {
            Color32::WHITE
        } else {
            ui.visuals().text_color()
        };

        ui.painter().rect(rect, 3.0, bg_color, stroke);

        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.center().x, rect.min.y + 14.0),
            egui::vec2(16.0, 16.0),
        );
        icon(which)
            .fit_to_exact_size(egui::vec2(16.0, 16.0))
            .tint(fg)
            .paint_at(ui, icon_rect);

        let galley = ui.painter().layout_no_wrap(label.to_owned(), font, fg);
        let pos = egui::pos2(
            rect.center().x - galley.size().x * 0.5,
            rect.min.y + 31.0 - galley.size().y * 0.5,
        );
        ui.painter().with_clip_rect(rect).galley(pos, galley, fg);
    }
    response
}

pub fn cmd_panel(app: &mut AssDrawApp, ctx: &egui::Context) {
    let lang = app.settings.language;
    let show_cmd_bar = app.show_cmd_bar;
    egui::TopBottomPanel::bottom("cmd_panel")
        .default_height(58.0)
        .show_animated(ctx, show_cmd_bar, |ui| {
            // Keep the box in sync with the drawing unless the user is typing in it.
            if !app.cmd_editing {
                app.sync_cmd();
            }

            ui.horizontal(|ui| {
                ui.label(RichText::new(t(lang, "cmd_title")).strong());

                let copy_btn = egui::Button::image_and_text(
                    themed(ui, Icon::Copy, 14.0),
                    RichText::new(t(lang, "cmd_copy")).strong(),
                );
                // Reserve room for the copy button first, then give the rest to the text box.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(copy_btn).clicked() {
                        app.copy_ass();
                    }
                    // Multi-line box (pasted multi-line ASS keeps its separators); Enter applies.
                    let enter = app.cmd_editing
                        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    let resp = ui.add_sized(
                        [ui.available_width().max(60.0), 22.0],
                        egui::TextEdit::multiline(&mut app.cmd_text)
                            .desired_rows(1)
                            .font(egui::TextStyle::Monospace),
                    );
                    app.cmd_editing = resp.has_focus();
                    if enter {
                        app.cmd_editing = false;
                        ui.memory_mut(|m| m.surrender_focus(resp.id));
                        app.apply_cmd_text();
                    }
                });
            });

            // Status bar row directly underneath
            ui.horizontal(|ui| {
                ui.label(format!("{:.0} {:.0}", app.last_cursor.0, app.last_cursor.1));
                if !app.status_cmd.is_empty() {
                    ui.separator();
                    ui.label(RichText::new(&app.status_cmd).color(Color32::from_rgb(0, 200, 255)).strong());
                } else if !app.status.is_empty() {
                    ui.separator();
                    ui.add(egui::Label::new(&app.status).truncate());
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let zoom_pct = (app.mouse.view_zoom * 100.0).round() as i64;
                    ui.label(format!("{zoom_pct}%"));
                });
            });
        });
}

const REPOS_ITEMS: [(ReposAnchor, &str, &str); 5] = [
    (ReposAnchor::Center, "pos_center", "5"),
    (ReposAnchor::TopLeft, "pos_tl", "7"),
    (ReposAnchor::TopRight, "pos_tr", "9"),
    (ReposAnchor::BottomLeft, "pos_bl", "1"),
    (ReposAnchor::BottomRight, "pos_br", "3"),
];

pub fn right_dock_panel(app: &mut AssDrawApp, ctx: &egui::Context) {
    let lang = app.settings.language;
    if !app.show_library && !app.show_settings {
        return;
    }

    // 1. Settings Dock (on the rightmost side)
    if app.show_settings {
        egui::SidePanel::right("settings_dock")
            .resizable(true)
            .default_width(270.0)
            .min_width(230.0)
            .max_width(380.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(RichText::new(t(lang, "dialog_settings_title")).strong().size(14.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✖").on_hover_text(t(lang, "btn_close")).clicked() {
                            app.show_settings = false;
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::CollapsingHeader::new(RichText::new(t(lang, "set_appearance")).strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            let col_w = (ui.available_width() * 0.55).max(80.0);
                            egui::Grid::new("settings_grid")
                                .num_columns(2)
                                .spacing([10.0, 4.0])
                                .max_col_width(col_w)
                                .show(ui, |ui| {
                                    ui.spacing_mut().slider_width = 70.0;
                                    color_picker_row(ui, t(lang, "menu_canvas"), &mut app.settings.canvas_bg);
                                    color_picker_row(ui, t(lang, "set_drawing"), &mut app.settings.shape_fill);
                                    grid_label(ui, t(lang, "set_drawing_op"));
                                    ui.add(egui::Slider::new(&mut app.settings.fill_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, t(lang, "rb_preview"), &mut app.settings.shape_preview);
                                    grid_label(ui, t(lang, "set_preview_op"));
                                    ui.add(egui::Slider::new(&mut app.settings.preview_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, t(lang, "set_outline"), &mut app.settings.editor_outline);
                                    grid_label(ui, t(lang, "set_outline_op"));
                                    ui.add(egui::Slider::new(&mut app.settings.editor_outline_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, t(lang, "set_ass_border"), &mut app.settings.shape_outline);
                                    grid_label(ui, t(lang, "set_ass_border_op"));
                                    ui.add(egui::Slider::new(&mut app.settings.outline_opacity, 0..=255));
                                    ui.end_row();

                                    grid_label(ui, t(lang, "set_outline_width"));
                                    ui.add(
                                        egui::DragValue::new(&mut app.settings.border_width)
                                            .range(0.0..=100.0)
                                            .speed(0.1),
                                    );
                                    ui.end_row();

                                    grid_label(ui, t(lang, "set_fill_enabled"));
                                    ui.checkbox(&mut app.settings.show_fill, "");
                                    ui.end_row();

                                    color_picker_row(ui, t(lang, "set_guides"), &mut app.settings.guideline_color);
                                    grid_label(ui, t(lang, "set_guides_op"));
                                    ui.add(egui::Slider::new(&mut app.settings.guideline_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, t(lang, "set_points"), &mut app.settings.point_main);
                                    grid_label(ui, t(lang, "set_points_op"));
                                    ui.add(egui::Slider::new(&mut app.settings.point_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, t(lang, "set_ctrl_points"), &mut app.settings.point_control);
                                    grid_label(ui, t(lang, "set_ctrl_points_op"));
                                    ui.add(egui::Slider::new(&mut app.settings.control_point_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, t(lang, "set_sel_points"), &mut app.settings.point_selected);
                                    grid_label(ui, t(lang, "set_sel_points_op"));
                                    ui.add(egui::Slider::new(&mut app.settings.selected_point_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, t(lang, "set_library"), &mut app.settings.library_bg);
                                    color_picker_row(ui, t(lang, "set_library_shapes"), &mut app.settings.library_shape_color);
                                    color_picker_row(ui, t(lang, "set_origin"), &mut app.settings.origin_color);
                                    grid_label(ui, t(lang, "set_origin_size"));
                                    ui.add(egui::Slider::new(&mut app.settings.origin_cross_size, 1..=30));
                                    ui.end_row();

                                    color_picker_row(ui, t(lang, "set_ruler_h"), &mut app.settings.ruler_h_color);
                                    color_picker_row(ui, t(lang, "set_ruler_v"), &mut app.settings.ruler_v_color);
                                });
                        });

                    egui::CollapsingHeader::new(RichText::new(t(lang, "set_behavior")).strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.checkbox(&mut app.settings.capitalize_cmds, t(lang, "set_capitalize"));
                            ui.checkbox(&mut app.settings.confirm_quit, t(lang, "set_confirm_quit"));
                            ui.checkbox(&mut app.settings.use_multicolor, t(lang, "set_multicolor"));
                            ui.checkbox(&mut app.settings.parse_spc, t(lang, "set_parse_spc"));
                            ui.checkbox(&mut app.settings.auto_ask_img_opacity, t(lang, "set_ask_img_opacity"));
                            ui.add(
                                egui::Slider::new(&mut app.settings.bg_alpha_default, 0..=255)
                                    .text(t(lang, "set_default_img_opacity")),
                            );
                            ui.checkbox(&mut app.settings.no_splash, t(lang, "set_no_splash"));
                        });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new(format!("  {}  ", t(lang, "btn_apply"))).strong()).clicked() {
                            app.settings.save();
                            app.settings_saved = app.settings.clone();
                            app.sync_cmd();
                            app.set_status(t(lang, "msg_settings_saved"));
                        }
                        if ui.button(format!(" {} ", t(lang, "set_revert"))).clicked() {
                            // Back to the last applied values (language and theme are saved at once).
                            let (language, dark, system) = (
                                app.settings.language,
                                app.settings.theme_dark,
                                app.settings.theme_system,
                            );
                            app.settings = app.settings_saved.clone();
                            app.settings.language = language;
                            app.settings.theme_dark = dark;
                            app.settings.theme_system = system;
                            app.sync_cmd();
                            app.set_status(t(lang, "msg_settings_reset"));
                        }
                    });
                });
            });
    }

    // 2. Shapes Library Dock (docked to the left of Settings dock, right of Canvas)
    if app.show_library {
        egui::SidePanel::right("library_dock")
            .resizable(true)
            .default_width(125.0)
            .min_width(95.0)
            .max_width(200.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(t(lang, "dialog_library_title")).strong().size(13.0),
                        )
                        .truncate(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✖").on_hover_text(t(lang, "btn_close")).clicked() {
                            app.show_library = false;
                        }
                    });
                });
                ui.horizontal_wrapped(|ui| {
                    let btn = |ui: &mut egui::Ui, which: Icon, tip: &'static str| {
                        let img = themed(ui, which, 13.0);
                        ui.add(egui::Button::image(img))
                            .on_hover_text(t(lang, tip))
                            .clicked()
                    };
                    if btn(ui, Icon::SquareCheck, "lib_select_all") {
                        app.library.select_all();
                    }
                    if btn(ui, Icon::Square, "lib_select_none") {
                        app.library.select_none();
                    }
                    if btn(ui, Icon::Trash, "lib_delete_sel") {
                        app.library.delete_selected();
                        app.library.save();
                        app.set_status(t(lang, "msg_shapes_deleted"));
                    }
                    if btn(ui, Icon::Plus, "lib_add_current") {
                        let name = format!("{} {}", t(lang, "lib_shape"), app.library.entries.len() + 1);
                        app.library.add_front(name, app.drawing.clone());
                        app.library.save();
                        app.set_status(t(lang, "msg_shape_added"));
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut lib_action: Option<(usize, LibAction)> = None;
                    for (idx, entry) in app.library.entries.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut entry.selected, "");
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::Vec2::new(ui.available_width(), 55.0),
                                egui::Sense::click(),
                            );
                            let painter = ui.painter_at(rect);
                            painter.rect_filled(rect, 2.0, app.settings.library_bg.to_egui());
                            if resp.hovered() {
                                painter.rect_stroke(
                                    rect,
                                    2.0,
                                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 150, 255)),
                                );
                            }
                            draw_fitted_thumbnail(
                                &painter,
                                rect,
                                &entry.drawing,
                                app.settings.library_shape_color.to_egui(),
                            );
                            // Name label drawn under the thumb rect
                            painter.text(
                                egui::Pos2::new(rect.center().x, rect.bottom() - 2.0),
                                egui::Align2::CENTER_BOTTOM,
                                &entry.name,
                                egui::FontId::proportional(10.0),
                                egui::Color32::from_gray(180),
                            );
                            if resp.double_clicked() {
                                lib_action = Some((idx, LibAction::Load));
                            }
                            resp.context_menu(|ui| {
                                if ui.button(t(lang, "lib_ctx_load")).clicked() {
                                    lib_action = Some((idx, LibAction::Load));
                                    ui.close_menu();
                                }
                                if ui.button(t(lang, "lib_ctx_copy")).clicked() {
                                    lib_action = Some((idx, LibAction::CopyAss));
                                    ui.close_menu();
                                }
                                if ui.button(t(lang, "lib_ctx_overwrite")).clicked() {
                                    lib_action = Some((idx, LibAction::Overwrite));
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button(t(lang, "lib_ctx_delete")).clicked() {
                                    lib_action = Some((idx, LibAction::Delete));
                                    ui.close_menu();
                                }
                            });
                        });
                        ui.add_space(2.0);
                    }
                    if let Some((idx, action)) = lib_action {
                        match action {
                            LibAction::Load => {
                                let entry = app.library.get(idx).map(|e| (e.name.clone(), e.drawing.clone()));
                                if let Some((name, dwg)) = entry {
                                    app.drawing.load_shape_from(&dwg);
                                    app.mouse.clear_selection();
                                    app.mouse.reset_transform();
                                    app.sync_cmd();
                                    app.set_status(format!("{name} {}", t(lang, "msg_loaded")));
                                }
                            }
                            LibAction::CopyAss => {
                                let style = app.settings.shape_style();
                                let line = app.library.get(idx).map(|e| {
                                    e.drawing.to_styled_ass(
                                        &style,
                                        app.settings.capitalize_cmds,
                                        app.settings.use_multicolor,
                                    )
                                });
                                if let Some(line) = line {
                                    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(line)) {
                                        Ok(()) => app.set_status(t(lang, "msg_copied")),
                                        Err(_) => app.set_status(t(lang, "msg_clip_error")),
                                    }
                                }
                            }
                            LibAction::Overwrite => {
                                let dwg = app.drawing.clone();
                                if let Some(e) = app.library.entries.get_mut(idx) {
                                    e.drawing = dwg;
                                    e.source_ass = e.drawing.to_ass_string();
                                    app.library.save();
                                    app.set_status(t(lang, "msg_shape_saved"));
                                }
                            }
                            LibAction::Delete => {
                                app.pending_lib_delete = Some(idx);
                            }
                        }
                    }
                });
            });
    }
}

/// A label that is cut off (with the full text on hover) instead of widening its container.
fn grid_label(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(text).truncate()).on_hover_text(text);
}

fn color_picker_row(ui: &mut egui::Ui, label: &str, color: &mut Color) {
    grid_label(ui, label);
    let mut egui_col = color.to_egui();
    let text = format!("({},{},{})", color.r, color.g, color.b);
    ui.horizontal(|ui| {
        if ui.color_edit_button_srgba(&mut egui_col).changed() {
            *color = Color::from_rgb(egui_col.r(), egui_col.g(), egui_col.b());
        }
        ui.label(RichText::new(text).monospace().size(11.0));
    });
    ui.end_row();
}

fn draw_fitted_thumbnail(
    painter: &egui::Painter,
    rect: egui::Rect,
    drawing: &Drawing,
    col: egui::Color32,
) {
    if drawing.commands.is_empty() || drawing.points.is_empty() {
        return;
    }
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in drawing.points.values() {
        if p.x < min_x { min_x = p.x; }
        if p.x > max_x { max_x = p.x; }
        if p.y < min_y { min_y = p.y; }
        if p.y > max_y { max_y = p.y; }
    }
    let w = (max_x - min_x).max(1.0);
    let h = (max_y - min_y).max(1.0);

    let pad = 8.0_f32;
    let avail_w = (rect.width() - pad * 2.0).max(1.0);
    let avail_h = (rect.height() - pad * 2.0).max(1.0);
    let scale = (avail_w / w as f32).min(avail_h / h as f32);

    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    let center = rect.center();

    let to_screen = |x: f64, y: f64| -> Pos2 {
        Pos2::new(
            center.x + ((x - cx) as f32) * scale,
            center.y + ((y - cy) as f32) * scale,
        )
    };

    let meshes = crate::canvas::fill::build_filled_meshes(drawing, &to_screen, col, 255);
    for m in meshes {
        painter.add(egui::Shape::mesh(m.mesh));
    }
}
