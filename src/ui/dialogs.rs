//! Dialogs and modal windows for ASSDraw-rs:
//! Matrix Transform (with 8 templates), Settings (Appearance & Behaviors),
//! Shapes Library, Manual & Shortcuts, About, Paste ASS, and Background Opacity.

use crate::app::AssDrawApp;
use crate::engine::transform::MatrixTransform;
use crate::i18n::t;
use egui::RichText;

pub fn dialogs(app: &mut AssDrawApp, ctx: &egui::Context) {
    let lang = app.settings.language;

    // 1. Matrix & Scale/Rotate Transform Dialog
    if app.show_transform {
        let mut open = true;
        let mut should_close = false;
        let mut do_apply = false;

        egui::Window::new(t(lang, "dialog_transform_title"))
            .open(&mut open)
            .resizable(true)
            .default_width(380.0)
            .show(ctx, |ui| {
                ui.heading(t(lang, "dialog_transform_title"));

                ui.group(|ui| {
                    ui.label(RichText::new(t(lang, "tmpl_title")).strong());
                    egui::ComboBox::from_label("")
                        .selected_text(app.transform_template_name.as_str())
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(false, t(lang, "tmpl_rot_90_cw")).clicked() {
                                app.active_matrix = MatrixTransform::template_rot_90_cw();
                                app.transform_template_name = t(lang, "tmpl_rot_90_cw").into();
                            }
                            if ui.selectable_label(false, t(lang, "tmpl_rot_90_ccw")).clicked() {
                                app.active_matrix = MatrixTransform::template_rot_90_ccw();
                                app.transform_template_name = t(lang, "tmpl_rot_90_ccw").into();
                            }
                            if ui.selectable_label(false, t(lang, "tmpl_rot_180")).clicked() {
                                app.active_matrix = MatrixTransform::template_rot_180();
                                app.transform_template_name = t(lang, "tmpl_rot_180").into();
                            }
                            if ui.selectable_label(false, t(lang, "tmpl_flip_h")).clicked() {
                                app.active_matrix = MatrixTransform::template_flip_h();
                                app.transform_template_name = t(lang, "tmpl_flip_h").into();
                            }
                            if ui.selectable_label(false, t(lang, "tmpl_flip_v")).clicked() {
                                app.active_matrix = MatrixTransform::template_flip_v();
                                app.transform_template_name = t(lang, "tmpl_flip_v").into();
                            }
                            if ui.selectable_label(false, t(lang, "tmpl_move_down")).clicked() {
                                app.active_matrix = MatrixTransform::template_move_down();
                                app.transform_template_name = t(lang, "tmpl_move_down").into();
                            }
                            if ui.selectable_label(false, t(lang, "tmpl_move_right")).clicked() {
                                app.active_matrix = MatrixTransform::template_move_right();
                                app.transform_template_name = t(lang, "tmpl_move_right").into();
                            }
                            if ui.selectable_label(false, t(lang, "tmpl_scale_h2")).clicked() {
                                app.active_matrix = MatrixTransform::template_scale_h2();
                                app.transform_template_name = t(lang, "tmpl_scale_h2").into();
                            }
                            if ui.selectable_label(false, t(lang, "tmpl_scale_v3")).clicked() {
                                app.active_matrix = MatrixTransform::template_scale_v3();
                                app.transform_template_name = t(lang, "tmpl_scale_v3").into();
                            }
                        });
                });

                ui.collapsing(t(lang, "matrix_params"), |ui| {
                    ui.label("x' = m11*(x-mx) + m12*(y-my) + nx");
                    ui.label("y' = m21*(x-mx) + m22*(y-my) + ny");
                    egui::Grid::new("matrix_grid").num_columns(4).show(ui, |ui| {
                        ui.label("m11:");
                        ui.add(egui::DragValue::new(&mut app.active_matrix.m11).speed(0.1));
                        ui.label("m12:");
                        ui.add(egui::DragValue::new(&mut app.active_matrix.m12).speed(0.1));
                        ui.end_row();

                        ui.label("m21:");
                        ui.add(egui::DragValue::new(&mut app.active_matrix.m21).speed(0.1));
                        ui.label("m22:");
                        ui.add(egui::DragValue::new(&mut app.active_matrix.m22).speed(0.1));
                        ui.end_row();

                        ui.label("mx:");
                        ui.add(egui::DragValue::new(&mut app.active_matrix.mx).speed(0.5));
                        ui.label("my:");
                        ui.add(egui::DragValue::new(&mut app.active_matrix.my).speed(0.5));
                        ui.end_row();

                        ui.label("nx:");
                        ui.add(egui::DragValue::new(&mut app.active_matrix.nx).speed(0.5));
                        ui.label("ny:");
                        ui.add(egui::DragValue::new(&mut app.active_matrix.ny).speed(0.5));
                        ui.end_row();
                    });
                });

                ui.group(|ui| {
                    ui.label(RichText::new(t(lang, "quick_sr")).strong());
                    ui.add(egui::Slider::new(&mut app.transform_scale, 0.1..=10.0).text(t(lang, "opt_scale")));
                    ui.add(egui::Slider::new(&mut app.transform_angle, -180.0..=180.0).text(t(lang, "angle")));
                });

                ui.checkbox(&mut app.transform_selected_only, t(lang, "selected_only"));

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(RichText::new(t(lang, "btn_apply")).strong()).clicked() {
                        do_apply = true;
                        should_close = true;
                    }
                    if ui.button(t(lang, "btn_cancel")).clicked() {
                        should_close = true;
                    }
                });
            });

        if do_apply {
            let do_sr = (app.transform_scale - 1.0).abs() > 0.001 || app.transform_angle.abs() > 0.001;
            // Identity matrix + scale/rotate → only scale/rotate (single history entry).
            let m = &app.active_matrix;
            let is_identity = (m.m11 - 1.0).abs() < 1e-12
                && m.m12.abs() < 1e-12
                && m.m21.abs() < 1e-12
                && (m.m22 - 1.0).abs() < 1e-12
                && m.mx.abs() < 1e-12
                && m.my.abs() < 1e-12
                && m.nx.abs() < 1e-12
                && m.ny.abs() < 1e-12;
            if !is_identity {
                app.apply_matrix();
            }
            if do_sr {
                app.apply_transform();
            }
            if is_identity && !do_sr {
                app.set_status(t(lang, "msg_trans_applied"));
            }
        }
        if !open || should_close {
            app.show_transform = false;
        }
    }



    // 4. Paste ASS Dialog
    if app.show_paste {
        let mut open = true;
        let mut should_apply = false;
        let mut should_close = false;

        egui::Window::new(t(lang, "dialog_paste_title"))
            .open(&mut open)
            .resizable(true)
            .default_width(500.0)
            .show(ctx, |ui| {
                ui.heading(t(lang, "dialog_paste_title"));
                ui.label(t(lang, "paste_hint"));
                ui.add(
                    egui::TextEdit::multiline(&mut app.paste_buffer)
                        .desired_rows(8)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal(|ui| {
                    if ui.button(RichText::new(t(lang, "btn_apply")).strong()).clicked() {
                        should_apply = true;
                        should_close = true;
                    }
                    if ui.button(t(lang, "btn_cancel")).clicked() {
                        should_close = true;
                    }
                });
            });

        if should_apply {
            app.apply_paste();
        }
        if !open || should_close {
            app.show_paste = false;
        }
    }

    // 5. Background Image Opacity Dialog
    if app.show_bg_alpha_dialog {
        let mut open = true;
        let mut should_close = false;

        egui::Window::new(t(lang, "action_bg_alpha"))
            .open(&mut open)
            .resizable(false)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.label(t(lang, "action_bg_alpha"));
                ui.add(egui::Slider::new(&mut app.bg_alpha, 0..=255).text("Alpha"));
                if ui.button(t(lang, "btn_close")).clicked() {
                    should_close = true;
                }
            });

        if !open || should_close {
            app.show_bg_alpha_dialog = false;
        }
    }

    // 6. User Manual & Shortcuts Dialog
    if app.show_manual {
        let mut open = true;

        egui::Window::new(t(lang, "dialog_manual_title"))
            .open(&mut open)
            .resizable(true)
            .default_width(650.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(t(lang, "man_title"));
                    ui.label(t(lang, "man_intro"));

                    ui.separator();
                    ui.label(RichText::new(t(lang, "man_h1")).strong());
                    ui.label(t(lang, "man_m"));
                    ui.label(t(lang, "man_n"));
                    ui.label(t(lang, "man_l"));
                    ui.label(t(lang, "man_b"));
                    ui.label(t(lang, "man_s"));
                    ui.label(t(lang, "man_p"));
                    ui.label(t(lang, "man_c"));

                    ui.separator();
                    ui.label(RichText::new(t(lang, "man_h2")).strong());
                    ui.label(t(lang, "man_left"));
                    ui.label(t(lang, "man_right"));
                    ui.label(t(lang, "man_wheel"));
                    ui.label(t(lang, "man_origin"));
                    ui.label(t(lang, "man_ctx"));

                    ui.separator();
                    ui.label(RichText::new(t(lang, "man_h3")).strong());
                    egui::Grid::new("shortcuts_grid").striped(true).show(ui, |ui| {
                        ui.label(RichText::new("F1").strong()); ui.label(t(lang, "man_k_select")); ui.end_row();
                        ui.label(RichText::new("F2 / M").strong()); ui.label(t(lang, "man_k_m")); ui.end_row();
                        ui.label(RichText::new("N").strong()); ui.label(t(lang, "man_k_n")); ui.end_row();
                        ui.label(RichText::new("F3 / L").strong()); ui.label(t(lang, "man_k_l")); ui.end_row();
                        ui.label(RichText::new("F4 / B").strong()); ui.label(t(lang, "man_k_b")); ui.end_row();
                        ui.label(RichText::new("S").strong()); ui.label(t(lang, "man_k_s")); ui.end_row();
                        ui.label(RichText::new("F5 / Del").strong()); ui.label(t(lang, "man_k_del")); ui.end_row();
                        ui.label(RichText::new("F6 / R").strong()); ui.label(t(lang, "man_k_sr")); ui.end_row();
                        ui.label(RichText::new("F7").strong()); ui.label(t(lang, "man_k_bil")); ui.end_row();
                        ui.label(RichText::new("F8").strong()); ui.label(t(lang, "man_k_col")); ui.end_row();
                        ui.label(RichText::new("Ctrl+Z / Ctrl+Y").strong()); ui.label(t(lang, "man_k_undo")); ui.end_row();
                        ui.label(RichText::new("Ctrl+C / Ctrl+V").strong()); ui.label(t(lang, "man_k_copy")); ui.end_row();
                        ui.label(RichText::new("Ctrl+I").strong()); ui.label(t(lang, "man_k_img")); ui.end_row();
                        ui.label(RichText::new("Ctrl+5 / 7 / 9 / 1 / 3").strong()); ui.label(t(lang, "man_k_align")); ui.end_row();
                        ui.label(RichText::new("Shift+F1 / F2 / F3").strong()); ui.label(t(lang, "man_k_pan")); ui.end_row();
                    });
                    ui.separator();
                    // Full HTML guide (replaces classic CHM for cross-platform)
                    if ui.button(t(lang, "help_open_guide")).clicked() {
                        open_html_guide();
                    }
                });
            });

        if !open {
            app.show_manual = false;
        }
    }

    // 7. About Dialog
    if app.show_about {
        let mut open = true;
        let mut should_close = false;

        egui::Window::new(t(lang, "dialog_about_title"))
            .open(&mut open)
            .resizable(false)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // Original About banner BMP
                    ui.add(
                        egui::Image::new(egui::include_image!("../../assets/bitmaps/assdraw3.bmp"))
                            .max_width(380.0),
                    );
                    ui.add_space(6.0);
                    ui.heading("ASSDraw3 v3.1 (Rust)");
                    ui.label(RichText::new("Aegisub & ASS Vector Graphic Editor").italics());
                    ui.add_space(6.0);
                    ui.label(t(lang, "about_desc"));
                    ui.add_space(4.0);
                    ui.label(RichText::new(t(lang, "about_authors")).strong());
                    ui.label(t(lang, "about_credit_orig"));
                    ui.label("• Thomas Goyne, MIRIMIRIM, biki-desu, amanosatosi");
                    ui.label(t(lang, "about_credit_port"));
                    ui.add_space(6.0);
                    ui.label(t(lang, "about_license"));
                    ui.hyperlink_to("GitHub: assdraw", "https://github.com/KerimDemirkaynak/assdraw");
                    ui.add_space(8.0);
                    if ui.button(RichText::new(format!("  {}  ", t(lang, "btn_close"))).strong()).clicked() {
                        should_close = true;
                    }
                    ui.add_space(4.0);
                });
            });

        if !open || should_close {
            app.show_about = false;
        }
    }
    // Contour colour dialog (double-click / Colour tool)
    if app.show_contour_color {
        let mut open = true;
        let idx = app.contour_color_idx;
        let title = format!("{} #{}", t(lang, "tool_coloring"), idx + 1);
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.label(t(lang, "dlg_contour_color"));
                let mut rgb = [
                    app.contour_color_edit.r,
                    app.contour_color_edit.g,
                    app.contour_color_edit.b,
                ];
                if ui.color_edit_button_srgb(&mut rgb).changed() {
                    app.contour_color_edit = crate::engine::Color::from_rgb(rgb[0], rgb[1], rgb[2]);
                }
                ui.horizontal(|ui| {
                    if ui.button(t(lang, "btn_apply")).clicked() {
                        app.drawing.push_history_desc("Contour color");
                        app.drawing
                            .set_contour_color(idx, app.contour_color_edit);
                        app.settings.shape_fill = app.contour_color_edit;
                        app.sync_cmd();
                        app.set_status(format!(
                            "{} #{}",
                            t(lang, "msg_color_set"),
                            idx + 1
                        ));
                        app.show_contour_color = false;
                    }
                    if ui.button(t(lang, "btn_close")).clicked() {
                        app.show_contour_color = false;
                    }
                });
            });
        if !open {
            app.show_contour_color = false;
        }
    }

}


/// Open packaged `docs/guide.html` in the system browser (CHM replacement).
fn open_html_guide() {
    let candidates = [
        std::path::PathBuf::from("docs/guide.html"),
        std::path::PathBuf::from("guide.html"),
        // next to the executable
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("docs/guide.html")))
            .unwrap_or_default(),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("guide.html")))
            .unwrap_or_default(),
    ];
    for path in candidates {
        if path.as_os_str().is_empty() {
            continue;
        }
        if path.is_file() {
            // `absolute` (unlike `canonicalize`) keeps a plain drive path on Windows, which `start`
            // understands; file:// URLs built from backslash paths do not work there.
            let abs = std::path::absolute(&path).unwrap_or(path);
            open_url(&abs.display().to_string());
            return;
        }
    }
    // Fallback: online-style relative won't work; still try cwd
    open_url("docs/guide.html");
}

fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}
