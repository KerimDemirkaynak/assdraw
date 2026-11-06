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
                    ui.label(RichText::new("Şablonlar (Templates)").strong());
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

                ui.collapsing("Matris Parametreleri (Matrix Values)", |ui| {
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
                    ui.label(RichText::new("Hızlı Ölçekle & Döndür (Scale & Rotate)").strong());
                    ui.add(egui::Slider::new(&mut app.transform_scale, 0.1..=10.0).text(t(lang, "opt_scale")));
                    ui.add(egui::Slider::new(&mut app.transform_angle, -180.0..=180.0).text("Açı (Angle °)"));
                });

                ui.checkbox(&mut app.transform_selected_only, "Yalnızca seçili noktaları dönüştür (Selected only)");

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
            app.apply_matrix();
            if (app.transform_scale - 1.0).abs() > 0.001 || app.transform_angle.abs() > 0.001 {
                app.apply_transform();
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
                ui.label("Aegisub veya ASS formatındaki çizim komutlarını buraya yapıştırın:");
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
                    ui.heading("ASSDraw-rs Kullanım Kılavuzu");
                    ui.label("ASSDraw-rs, Aegisub ve Advanced SubStation Alpha (.ass) altyazı dosyaları için vektör çizim tasarlama aracıdır.");

                    ui.separator();
                    ui.label(RichText::new("1. Temel Çizim Komutları").strong());
                    ui.label("• M x y: Sanal kalemi yeni koordinata taşır ve mevcut konturu kapatır (Move to & close).");
                    ui.label("• N x y: Konturu kapatmadan sanal kalemi yeni koordinata taşır (Move to without closing).");
                    ui.label("• L x y: Düz çizgi çizer (Line to). Art arda koordinat çiftleri alır.");
                    ui.label("• B c1x c1y c2x c2y x y: Üçüncü derece Bezier eğrisi çizer (Cubic Bezier curve).");
                    ui.label("• S x1 y1 x2 y2 ...: B-Spline eğrisi çizer.");
                    ui.label("• P x y: Spline eğrisine yeni nokta ekleyerek uzatır.");
                    ui.label("• C: Spline eğrisini kapatır (Close).");

                    ui.separator();
                    ui.label(RichText::new("2. Fare & Tuval Kullanımı").strong());
                    ui.label("• Sol Tık & Sürükleme: Noktaları taşır veya boş alanda çerçeve ile çoklu seçim yapar.");
                    ui.label("• Sağ Tuşla Sürükleme: Tuvali serbestçe kaydırır (Pan).");
                    ui.label("• Fare Tekerleği: Yakınlaştırır ve uzaklaştırır (Zoom in/out).");
                    ui.label("• [0,0] Başlangıç Noktası: Kırmızı artı simgesi sürüklenerek koordinat merkezi ayarlanabilir.");
                    ui.label("• Sağ Tık Menüsü: Çizgi↔Bezier dönüşümü, C1 yumuşak bağlantı ve [0,0] hizalama.");

                    ui.separator();
                    ui.label(RichText::new("3. Klavye Kısayolları").strong());
                    egui::Grid::new("shortcuts_grid").striped(true).show(ui, |ui| {
                        ui.label(RichText::new("F1").strong()); ui.label("Seçim / Sürükleme Kipi"); ui.end_row();
                        ui.label(RichText::new("F2 / M").strong()); ui.label("Çiz M (Kapat & Taşı)"); ui.end_row();
                        ui.label(RichText::new("N").strong()); ui.label("Çiz N (Kapatmadan Taşı)"); ui.end_row();
                        ui.label(RichText::new("F3 / L").strong()); ui.label("Çiz L (Düz Çizgi)"); ui.end_row();
                        ui.label(RichText::new("F4 / B").strong()); ui.label("Çiz B (Bezier Eğrisi)"); ui.end_row();
                        ui.label(RichText::new("F5 / S").strong()); ui.label("Çiz S (Spline)"); ui.end_row();
                        ui.label(RichText::new("Del").strong()); ui.label("Seçili Noktaları Sil"); ui.end_row();
                        ui.label(RichText::new("F6 / R").strong()); ui.label("Ölçekle / Döndür"); ui.end_row();
                        ui.label(RichText::new("F7").strong()); ui.label("Bilineer Dönüşüm"); ui.end_row();
                        ui.label(RichText::new("F8").strong()); ui.label("Renklendirme Kipi"); ui.end_row();
                        ui.label(RichText::new("Ctrl+Z / Ctrl+Y").strong()); ui.label("Geri Al / Yinele"); ui.end_row();
                        ui.label(RichText::new("Ctrl+C / Ctrl+V").strong()); ui.label("ASS Kopyala / Yapıştır"); ui.end_row();
                        ui.label(RichText::new("Ctrl+5 / 7 / 9 / 1 / 3").strong()); ui.label("Şekli Merkez / Köşelere Hizala"); ui.end_row();
                        ui.label(RichText::new("Shift+F1 / F2 / F3").strong()); ui.label("Çizim / Arka Plan Kaydırma Kipi"); ui.end_row();
                    });
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
                    ui.add(egui::Image::new(egui::include_image!("../../assets/bitmaps/assdraw3.bmp")));
                    ui.add_space(6.0);
                    ui.heading("ASSDraw3 v3.1 (Rust)");
                    ui.label(RichText::new("Aegisub & ASS Vector Graphic Editor").italics());
                    ui.add_space(6.0);
                    ui.label("ASSDraw, ASS altyazıları için vektör şekil tasarım aracıdır.");
                    ui.add_space(4.0);
                    ui.label(RichText::new("Yazarlar & Katkıda Bulunanlar:").strong());
                    ui.label("• ai-chan (ASSDraw3 Orijinal Tasarım & Kod)");
                    ui.label("• Thomas Goyne, MIRIMIRIM, biki-desu, amanosatosi");
                    ui.label("• Kerim Demirkaynak (Modern Rust Port)");
                    ui.add_space(6.0);
                    ui.label("Lisans: BSD 3-Clause License");
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
}
