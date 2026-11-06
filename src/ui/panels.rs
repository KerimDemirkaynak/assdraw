//! Application Panels: Top Menu Bar, Tool Ribbon, Bottom Command Editor, and Right Dock Panel.
//! Faithfully mirrors original ASSDraw3 layout (Image 3) with SVG icons.

use crate::app::AssDrawApp;
use crate::canvas::mouse::DragTarget;
use crate::engine::transform::ReposAnchor;
use crate::engine::{Color, Drawing};
use crate::i18n::{t, Language};
use crate::ui::Tool;
use egui::{Color32, Pos2, RichText};

pub fn top_bar(app: &mut AssDrawApp, ctx: &egui::Context) {
    let lang = app.settings.language;

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // 1. Tuval (Canvas)
            ui.menu_button("Tuval", |ui| {
                if ui.button(format!("{} (Ctrl+N)", t(lang, "action_clear"))).clicked() {
                    app.clear();
                    ui.close_menu();
                }
                if ui.button(format!("{} (Ctrl+P)", t(lang, "action_preview"))).clicked() {
                    app.mouse.preview_mode = !app.mouse.preview_mode;
                    ui.close_menu();
                }
                if ui.button("Dönüştür...").clicked() {
                    app.show_transform = true;
                    ui.close_menu();
                }
                if ui.button("Panodan Yapıştır (Ctrl+V)").clicked() {
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
                if ui.button("Aç (JSON)...").clicked() {
                    app.load_json();
                    ui.close_menu();
                }
                if ui.button("Kaydet (JSON)...").clicked() {
                    app.save_json();
                    ui.close_menu();
                }
            });

            // 2. Kip (Mode)
            ui.menu_button("Kip", |ui| {
                if ui.radio(app.tool == Tool::Select, "Sürükle\tF1").clicked() {
                    app.tool = Tool::Select;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::MoveTo, "Çiz M\tF2").clicked() {
                    app.tool = Tool::MoveTo;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::LineTo, "Çiz L\tF3").clicked() {
                    app.tool = Tool::LineTo;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::Bezier, "Çiz B\tF4").clicked() {
                    app.tool = Tool::Bezier;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::Delete, "Sil\tF5").clicked() {
                    app.tool = Tool::Delete;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::ScaleRotate, "Ölçekle / Döndür\tF6").clicked() {
                    app.tool = Tool::ScaleRotate;
                    app.show_transform = true;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::Bilinear, "Bilineer Dönüşüm\tF7").clicked() {
                    app.tool = Tool::Bilinear;
                    ui.close_menu();
                }
                if ui.radio(app.tool == Tool::ColorPick, "Renklendirme").clicked() {
                    app.tool = Tool::ColorPick;
                    ui.close_menu();
                }
            });

            // 3. Arka plan (Background)
            ui.menu_button("Arka plan", |ui| {
                if ui.radio(app.mouse.drag_target == DragTarget::Drawing, "Çizimi Kaydır / Yakınlaştır\tShift+F1").clicked() {
                    app.mouse.drag_target = DragTarget::Drawing;
                    ui.close_menu();
                }
                if ui.radio(app.mouse.drag_target == DragTarget::Background, "Arka Planı Kaydır / Yakınlaştır\tShift+F2").clicked() {
                    app.mouse.drag_target = DragTarget::Background;
                    ui.close_menu();
                }
                if ui.radio(app.mouse.drag_target == DragTarget::Both, "Her İkisini Kaydır\tShift+F3").clicked() {
                    app.mouse.drag_target = DragTarget::Both;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Panodan Resim Yapıştır (Ctrl+V)").clicked() {
                    app.paste_from_clipboard(ctx);
                    ui.close_menu();
                }
                if ui.button("Resim Dosyası Yükle...").clicked() {
                    app.load_bg(ctx);
                    ui.close_menu();
                }
                if ui.button("Arka Plan Saydamlığı...").clicked() {
                    app.show_bg_alpha_dialog = true;
                    ui.close_menu();
                }
                if app.bg_image.is_some() && ui.button("Arka Planı Kaldır\tShift+Del").clicked() {
                    app.bg_image = None;
                    app.bg_path = None;
                    app.set_status(t(lang, "msg_bg_removed"));
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Yeniden Konumlandır [0, 0]", |ui| {
                    if ui.button("Merkez\tCtrl+Shift+5").clicked() {
                        app.mouse.bg_pan = egui::Vec2::ZERO;
                        ui.close_menu();
                    }
                });
            });

            // 4. Çalışma alanı (Workspace)
            ui.menu_button("Çalışma alanı", |ui| {
                if ui.checkbox(&mut app.show_library, "Şekil Kitaplığı").clicked() {
                    ui.close_menu();
                }
                if ui.checkbox(&mut app.show_settings, "Ayarlar").clicked() {
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Tema", |ui| {
                    if ui.radio(!app.settings.theme_dark, "Açık (Light)").clicked() {
                        app.settings.theme_dark = false;
                        ctx.set_visuals(egui::Visuals::light());
                        ui.close_menu();
                    }
                    if ui.radio(app.settings.theme_dark, "Koyu (Dark)").clicked() {
                        app.settings.theme_dark = true;
                        ctx.set_visuals(egui::Visuals::dark());
                        ui.close_menu();
                    }
                });
                ui.separator();
                ui.menu_button("Yeniden Konumlandır [0, 0]", |ui| {
                    if ui.button("Merkez\tCtrl+5").clicked() {
                        app.drawing.push_history_desc("Center shape");
                        app.drawing.reposition_shape(ReposAnchor::Center);
                        app.sync_cmd();
                        ui.close_menu();
                    }
                    if ui.button("Sol Üst\tCtrl+7").clicked() {
                        app.drawing.push_history_desc("Top-Left shape");
                        app.drawing.reposition_shape(ReposAnchor::TopLeft);
                        app.sync_cmd();
                        ui.close_menu();
                    }
                    if ui.button("Sağ Üst\tCtrl+9").clicked() {
                        app.drawing.push_history_desc("Top-Right shape");
                        app.drawing.reposition_shape(ReposAnchor::TopRight);
                        app.sync_cmd();
                        ui.close_menu();
                    }
                    if ui.button("Sol Alt\tCtrl+1").clicked() {
                        app.drawing.push_history_desc("Bottom-Left shape");
                        app.drawing.reposition_shape(ReposAnchor::BottomLeft);
                        app.sync_cmd();
                        ui.close_menu();
                    }
                    if ui.button("Sağ Alt\tCtrl+3").clicked() {
                        app.drawing.push_history_desc("Bottom-Right shape");
                        app.drawing.reposition_shape(ReposAnchor::BottomRight);
                        app.sync_cmd();
                        ui.close_menu();
                    }
                });
                ui.separator();
                if ui.button("Çalışma Alanını Sıfırla").clicked() {
                    app.mouse.pan = egui::Vec2::ZERO;
                    app.mouse.view_zoom = 2.0;
                    app.mouse.bg_pan = egui::Vec2::ZERO;
                    app.mouse.bg_zoom = 1.0;
                    ui.close_menu();
                }
            });

            // 5. Yardım (Help)
            ui.menu_button("Yardım", |ui| {
                if ui.button("Kullanım Kılavuzu").clicked() {
                    app.show_manual = true;
                    ui.close_menu();
                }
                if ui.button("Hakkında").clicked() {
                    app.show_about = true;
                    ui.close_menu();
                }
            });

            // 6. Dil (Language)
            ui.menu_button("Dil", |ui| {
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

    // Top Tool Ribbon - Matching ASSDraw3 (Image 3) with SVGs
    egui::TopBottomPanel::top("tool_ribbon").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // 1. Temizle (broom.svg)
            if ribbon_tool_btn(
                ui,
                egui::include_image!("../../assets/svg/solid/broom.svg"),
                "Temizle",
                false,
            )
            .on_hover_text("Tuvali temizle (Ctrl+N)")
            .clicked()
            {
                app.clear();
            }

            // 2. Önizleme (eye.svg)
            if ribbon_tool_btn(
                ui,
                egui::include_image!("../../assets/svg/solid/eye.svg"),
                "Önizleme",
                app.mouse.preview_mode,
            )
            .on_hover_text("Önizleme kipini aç/kapat")
            .clicked()
            {
                app.mouse.preview_mode = !app.mouse.preview_mode;
            }

            // 3. Renklendirme (palette.svg)
            if ribbon_tool_btn(
                ui,
                egui::include_image!("../../assets/svg/solid/palette.svg"),
                "Renklendirme",
                app.tool == Tool::ColorPick,
            )
            .on_hover_text("Renk seç / Renklendirme")
            .clicked()
            {
                app.tool = if app.tool == Tool::ColorPick {
                    Tool::Select
                } else {
                    Tool::ColorPick
                };
            }

            // 4. Zoom Slider without cluttering label (zoom % is displayed on the bottom status bar)
            ui.add_sized(
                [90.0, 18.0],
                egui::Slider::new(&mut app.mouse.view_zoom, 0.2..=10.0).show_value(false),
            );

            ui.separator();

            // 5. Drawing Mode Buttons (Sürükle, Taşı, Doğru, Bezier, Sil)
            let group1 = [
                (
                    Tool::Select,
                    "Sürükle",
                    egui::include_image!("../../assets/svg/solid/arrow-pointer.svg"),
                ),
                (
                    Tool::MoveTo,
                    "Taşı",
                    egui::include_image!("../../assets/svg/solid/location-dot.svg"),
                ),
                (
                    Tool::LineTo,
                    "Doğru",
                    egui::include_image!("../../assets/svg/solid/slash.svg"),
                ),
                (
                    Tool::Bezier,
                    "Bezier",
                    egui::include_image!("../../assets/svg/solid/bezier-curve.svg"),
                ),
                (
                    Tool::Delete,
                    "Sil",
                    egui::include_image!("../../assets/svg/solid/eraser.svg"),
                ),
            ];
            for (t_item, label, img_src) in group1 {
                if ribbon_tool_btn(ui, img_src, label, app.tool == t_item).clicked() {
                    app.tool = t_item;
                }
            }

            ui.separator();

            // 6. Transforms (Ölçekle/Döndür, Bilineer)
            if ribbon_tool_btn(
                ui,
                egui::include_image!("../../assets/svg/solid/arrows-rotate.svg"),
                "Ölçekle/Döndür",
                app.tool == Tool::ScaleRotate,
            )
            .clicked()
            {
                app.tool = Tool::ScaleRotate;
                app.show_transform = true;
            }

            if ribbon_tool_btn(
                ui,
                egui::include_image!("../../assets/svg/solid/draw-polygon.svg"),
                "Bilineer",
                app.tool == Tool::Bilinear,
            )
            .clicked()
            {
                app.tool = Tool::Bilinear;
            }

            ui.separator();

            // 7. Renklendirme Tool
            if ribbon_tool_btn(
                ui,
                egui::include_image!("../../assets/svg/solid/palette.svg"),
                "Renklendirme",
                app.tool == Tool::ColorPick,
            )
            .clicked()
            {
                app.tool = Tool::ColorPick;
            }
        });
    });
}

fn ribbon_tool_btn(
    ui: &mut egui::Ui,
    img_src: egui::ImageSource<'_>,
    label: &str,
    is_active: bool,
) -> egui::Response {
    let desired_size = egui::vec2(52.0, 42.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let bg_color = if is_active {
            egui::Color32::from_rgb(0, 114, 206)
        } else if response.hovered() {
            visuals.bg_fill
        } else {
            egui::Color32::TRANSPARENT
        };
        let stroke = if is_active {
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(140, 190, 255))
        } else if response.hovered() {
            visuals.bg_stroke
        } else {
            egui::Stroke::NONE
        };

        ui.painter().rect(rect, 3.0, bg_color, stroke);

        // Icon on top
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.center().x, rect.min.y + 14.0),
            egui::vec2(16.0, 16.0),
        );
        let img = egui::Image::new(img_src).fit_to_exact_size(egui::vec2(16.0, 16.0));
        img.paint_at(ui, icon_rect);

        // Label on bottom
        let text_color = if is_active {
            egui::Color32::WHITE
        } else {
            visuals.text_color()
        };
        ui.painter().text(
            egui::pos2(rect.center().x, rect.min.y + 31.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            text_color,
        );
    }
    response
}

pub fn cmd_panel(app: &mut AssDrawApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("cmd_panel")
        .default_height(58.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Çizim komutları").strong());

                let mut text = if app.settings.use_multicolor {
                    app.drawing.to_multicolor_ass(
                        &app.settings.shape_fill,
                        &app.settings.shape_outline,
                        app.settings.border_width,
                        app.settings.capitalize_cmds,
                    )
                } else {
                    app.drawing.to_full_ass_line(
                        &app.settings.shape_fill,
                        &app.settings.shape_outline,
                        app.settings.border_width,
                        app.settings.capitalize_cmds,
                    )
                };

                let resp = ui.add(
                    egui::TextEdit::singleline(&mut text)
                        .desired_width(ui.available_width() - 140.0)
                        .font(egui::TextStyle::Monospace),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    app.paste_buffer = text;
                    app.drawing.push_history_desc("Edit ASS line");
                    app.drawing.clear();
                    app.apply_paste();
                }

                let img_copy = egui::Image::new(egui::include_image!("../../assets/svg/regular/copy.svg"))
                    .fit_to_exact_size(egui::vec2(14.0, 14.0));
                if ui.add(egui::Button::image_and_text(img_copy, RichText::new("ASS kopyala").strong())).clicked() {
                    app.copy_ass();
                }
            });

            // Status bar row directly underneath
            ui.horizontal(|ui| {
                ui.label(format!("{:.0} {:.0}", app.last_cursor.0, app.last_cursor.1));
                if !app.status_cmd.is_empty() {
                    ui.separator();
                    ui.label(RichText::new(&app.status_cmd).color(Color32::from_rgb(0, 200, 255)).strong());
                } else if !app.status.is_empty() {
                    ui.separator();
                    ui.label(&app.status);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let zoom_pct = (app.mouse.view_zoom * 500.0).round() as i64;
                    ui.label(format!("{zoom_pct}%"));
                });
            });
        });
}

pub fn right_dock_panel(app: &mut AssDrawApp, ctx: &egui::Context) {
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
                    ui.heading(RichText::new("Ayarlar").strong().size(14.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖").on_hover_text("Kapat").clicked() {
                            app.show_settings = false;
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::CollapsingHeader::new(RichText::new("Görünüm").strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::Grid::new("settings_grid")
                                .num_columns(2)
                                .spacing([10.0, 4.0])
                                .show(ui, |ui| {
                                    color_picker_row(ui, "Tuval", &mut app.settings.canvas_bg);
                                    color_picker_row(ui, "Çizim", &mut app.settings.shape_fill);
                                    ui.label("Çizim opaklığı");
                                    ui.add(egui::Slider::new(&mut app.settings.fill_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, "Önizleme", &mut app.settings.shape_preview);
                                    ui.label("Önizleme opaklığı");
                                    ui.add(egui::Slider::new(&mut app.settings.preview_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, "Kenarlık", &mut app.settings.shape_outline);
                                    ui.label("Kenarlık opaklığı");
                                    ui.add(egui::Slider::new(&mut app.settings.outline_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, "Kontrol çizgileri", &mut app.settings.guideline_color);
                                    ui.label("Kontrol çizgilerinin opaklığı");
                                    ui.add(egui::Slider::new(&mut app.settings.guideline_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, "Noktalar", &mut app.settings.point_main);
                                    ui.label("Nokta opaklığı");
                                    ui.add(egui::Slider::new(&mut app.settings.point_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, "Kontrol noktaları", &mut app.settings.point_control);
                                    ui.label("Kontrol noktalarının opaklığı");
                                    ui.add(egui::Slider::new(&mut app.settings.control_point_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, "Seçili noktalar", &mut app.settings.point_selected);
                                    ui.label("Seçili noktaların opaklığı");
                                    ui.add(egui::Slider::new(&mut app.settings.selected_point_opacity, 0..=255));
                                    ui.end_row();

                                    color_picker_row(ui, "Kitaplık", &mut app.settings.library_bg);
                                    color_picker_row(ui, "Kitaplık şekilleri", &mut app.settings.library_shape_color);
                                    color_picker_row(ui, "Başlangıç noktası", &mut app.settings.origin_color);
                                    ui.label("Başlangıç işaretinin boyutu");
                                    ui.add(egui::Slider::new(&mut app.settings.origin_cross_size, 1..=30));
                                    ui.end_row();

                                    color_picker_row(ui, "Yatay cetvel", &mut app.settings.ruler_h_color);
                                    color_picker_row(ui, "Dikey cetvel", &mut app.settings.ruler_v_color);
                                });
                        });

                    egui::CollapsingHeader::new(RichText::new("Davranışlar").strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.checkbox(&mut app.settings.capitalize_cmds, "Komut harflerini büyüt (M, L, B...)");
                            ui.checkbox(&mut app.settings.auto_bezier, "Otomatik Bezier");
                            ui.add(egui::Slider::new(&mut app.settings.bezier_smoothness, 0.1..=1.0).text("Düzgünlük"));
                            ui.checkbox(&mut app.settings.use_multicolor, "Çoklu renk (Multicolor ASS)");
                        });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("  Uygula  ").strong()).clicked() {
                            app.settings.save();
                            app.sync_cmd();
                            app.set_status("Ayarlar kaydedildi.");
                        }
                        if ui.button(" Geri döndür ").clicked() {
                            app.settings = crate::settings::AppSettings::default();
                            app.sync_cmd();
                            app.set_status("Ayarlar varsayılana döndürüldü.");
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
                    ui.heading(RichText::new("Şekil Kitaplığı").strong().size(13.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖").on_hover_text("Kapat").clicked() {
                            app.show_library = false;
                        }
                        let img_trash = egui::Image::new(egui::include_image!("../../assets/svg/solid/trash-can.svg"))
                            .fit_to_exact_size(egui::vec2(12.0, 12.0));
                        if ui.add(egui::Button::image_and_text(img_trash, "")).on_hover_text("Seçili Şekilleri Sil").clicked() {
                            app.library.delete_selected();
                            app.set_status("Seçili şekiller silindi.");
                        }
                        let img_uncheck = egui::Image::new(egui::include_image!("../../assets/svg/regular/square.svg"))
                            .fit_to_exact_size(egui::vec2(12.0, 12.0));
                        if ui.add(egui::Button::image_and_text(img_uncheck, "")).on_hover_text("Seçimleri Kaldır").clicked() {
                            app.library.select_none();
                        }
                        let img_check = egui::Image::new(egui::include_image!("../../assets/svg/regular/square-check.svg"))
                            .fit_to_exact_size(egui::vec2(12.0, 12.0));
                        if ui.add(egui::Button::image_and_text(img_check, "")).on_hover_text("Tümünü Seç").clicked() {
                            app.library.select_all();
                        }
                        let img_plus = egui::Image::new(egui::include_image!("../../assets/svg/solid/plus.svg"))
                            .fit_to_exact_size(egui::vec2(12.0, 12.0));
                        if ui.add(egui::Button::image_and_text(img_plus, "")).on_hover_text("Mevcut Çizimi Kitaplığa Ekle").clicked() {
                            let name = format!("Şekil {}", app.library.entries.len() + 1);
                            app.library.add(name, app.drawing.clone());
                            app.set_status("Şekil kitaplığa eklendi.");
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut to_load = None;
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
                            if resp.clicked() {
                                to_load = Some(idx);
                            }
                        });
                        ui.add_space(2.0);
                    }
                    if let Some(idx) = to_load {
                        if let Some(e) = app.library.get(idx) {
                            let name = e.name.clone();
                            let dwg = e.drawing.clone();
                            app.drawing = dwg;
                            app.sync_cmd();
                            app.set_status(format!("{} yüklendi", name));
                        }
                    }
                });
            });
    }
}

fn color_picker_row(ui: &mut egui::Ui, label: &str, color: &mut Color) {
    ui.label(label);
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
