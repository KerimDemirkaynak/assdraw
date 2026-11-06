//! egui central panel — wires CanvasMouseState to Drawing (canvas.cpp behaviour).
//! Supports high-DPI scaling, drag-and-drop background image, rulers, origin drag,
//! and complete context menu.

use crate::app::AssDrawApp;
use crate::canvas::fill::build_filled_meshes;
use crate::canvas::mouse::{
    hit_radius_logical, CanvasMouseState, DragTarget, LeftDragKind, MouseFrameResult, SelectMode,
};
use crate::canvas::render;
use crate::engine::Drawing;
use crate::i18n::t;
use crate::ui::Tool;
use egui::{Pos2, Sense, Vec2};

pub fn canvas(app: &mut AssDrawApp, ctx: &egui::Context) {
    app.mouse.tool = app.tool;

    // Handle drag-and-drop of image files
    ctx.input(|i| {
        for file in &i.raw.dropped_files {
            if let Some(path) = &file.path {
                app.load_bg_from_path(path, ctx);
                break;
            } else if let Some(bytes) = &file.bytes {
                app.load_bg_from_bytes(bytes, ctx);
                break;
            }
        }
    });

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
            let rect = resp.rect;
            let origin = rect.center() + app.mouse.pan;

            // Background canvas fill
            painter.rect_filled(rect, 0.0, app.settings.canvas_bg.to_egui());

            // Tracing Background Image
            if let Some(tex) = &app.bg_image {
                let bg_scale = app.bg_scale * app.mouse.bg_zoom;
                let size = tex.size_vec2() * bg_scale;
                let bg_origin = origin + app.mouse.bg_pan;
                let img_rect = egui::Rect::from_center_size(bg_origin, size);
                painter.image(
                    tex.id(),
                    img_rect,
                    egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    egui::Color32::from_white_alpha(app.bg_alpha),
                );
            }

            let scale = app.drawing.point_system.scale as f32 * app.mouse.view_zoom;

            // Grid
            if app.settings.show_grid && !app.mouse.preview_mode {
                render::draw_grid(&painter, rect, origin, scale);
            }

            let to_screen = |lx: f64, ly: f64| -> Pos2 {
                Pos2::new(origin.x + lx as f32 * scale, origin.y + ly as f32 * scale)
            };
            let to_logical = |p: Pos2| -> (f64, f64) {
                (
                    ((p.x - origin.x) / scale) as f64,
                    ((p.y - origin.y) / scale) as f64,
                )
            };

            // Process Mouse & Keyboard inputs on canvas
            let result = process_input(
                &mut app.mouse,
                &mut app.drawing,
                &app.settings,
                &resp,
                origin,
                scale,
                &to_logical,
            );

            app.last_cursor = result.status_coords;
            app.status_cmd = result.status_cmd.clone();

            if result.request_delete_sel {
                app.delete_sel();
            }
            if result.cmd_changed {
                app.sync_cmd();
            }

            if result.request_color_dialog {
                let col = app.settings.shape_fill;
                app.drawing.set_contour_color(result.contour_for_color, col);
                app.set_status(format!(
                    "{} #{}",
                    t(app.settings.language, "msg_color_set"),
                    result.contour_for_color + 1
                ));
            }

            // Sync selection to app state
            app.selected = app.mouse.selected.clone();

            // Render Filled Shapes (AGG-like Even-Odd Tessellation)
            if app.settings.show_fill {
                let fill_color = app.settings.shape_fill.to_egui();
                let meshes = build_filled_meshes(
                    &app.drawing,
                    &to_screen,
                    fill_color,
                    app.settings.fill_opacity,
                );
                for fs in meshes {
                    painter.add(egui::Shape::mesh(fs.mesh));
                }
            }

            // Render Stroke paths
            render::draw_paths(
                &painter,
                &app.drawing,
                &to_screen,
                app.settings.shape_fill,
                app.settings.shape_outline,
                app.settings.border_width as f32,
            );

            // Render Origin Crosshair [0,0]
            if !app.mouse.preview_mode {
                render::draw_origin(
                    &painter,
                    origin,
                    app.settings.origin_cross_size as f32,
                    app.settings.origin_color.to_egui(),
                    app.mouse.hover_origin,
                );
            }

            // Render Points, Bezier control handles & Marquee
            if app.settings.show_points && !app.mouse.preview_mode {
                render::draw_points(&painter, &app.drawing, &app.mouse, &to_screen, &app.settings);
            }

            // Rulers
            if app.settings.show_rulers && !app.mouse.preview_mode {
                render::draw_rulers(
                    &painter,
                    rect,
                    origin,
                    scale,
                    app.settings.ruler_h_color.to_egui(),
                );
            }
        });
}

fn process_input(
    mouse: &mut CanvasMouseState,
    drawing: &mut Drawing,
    settings: &crate::settings::AppSettings,
    resp: &egui::Response,
    origin: Pos2,
    scale: f32,
    to_logical: &dyn Fn(Pos2) -> (f64, f64),
) -> MouseFrameResult {
    let mut out = MouseFrameResult::default();
    let lang = settings.language;

    if mouse.preview_mode {
        return out;
    }

    // Mouse Wheel Zoom
    let scroll = resp.ctx.input(|i| i.smooth_scroll_delta.y);
    if scroll != 0.0 && resp.hovered() {
        let factor = if scroll > 0.0 { 1.12 } else { 1.0 / 1.12 };
        match mouse.drag_target {
            DragTarget::Drawing => {
                mouse.view_zoom = (mouse.view_zoom * factor).clamp(0.1, 35.0);
            }
            DragTarget::Background => {
                mouse.bg_zoom = (mouse.bg_zoom * factor).clamp(0.1, 35.0);
            }
            DragTarget::Both => {
                mouse.view_zoom = (mouse.view_zoom * factor).clamp(0.1, 35.0);
                mouse.bg_zoom = (mouse.bg_zoom * factor).clamp(0.1, 35.0);
            }
        }
    }

    let pointer = resp.hover_pos().or(resp.interact_pointer_pos());
    let (lx, ly) = pointer.map(to_logical).unwrap_or((0.0, 0.0));
    out.status_coords = (lx, ly);

    let hr = hit_radius_logical(drawing.point_system.scale as f32, mouse.view_zoom);

    // Origin hit test
    if let Some(pos) = pointer {
        let dist_to_orig = (pos.x - origin.x).hypot(pos.y - origin.y);
        mouse.hover_origin = dist_to_orig <= (settings.origin_cross_size as f32 * 1.2);
    } else {
        mouse.hover_origin = false;
    }

    // Hover Highlight
    if !matches!(mouse.left_drag, LeftDragKind::MovePoints { .. }) {
        mouse.pointed_id = CanvasMouseState::find_point_at(drawing, lx, ly, hr);
        if let Some(id) = mouse.pointed_id {
            mouse.status_cmd = CanvasMouseState::command_string_for_point(drawing, id);
            out.status_cmd = mouse.status_cmd.clone();
        } else {
            mouse.status_cmd.clear();
        }
    }

    // Right / Middle button panning
    let right_down = resp.ctx.input(|i| i.pointer.secondary_down());
    let middle_down = resp.ctx.input(|i| i.pointer.middle_down());

    if right_down || middle_down || mouse.tool == Tool::Pan {
        if resp.dragged() {
            let delta = resp.drag_delta();
            match mouse.drag_target {
                DragTarget::Drawing => {
                    mouse.pan += delta;
                }
                DragTarget::Background => {
                    mouse.bg_pan += delta;
                }
                DragTarget::Both => {
                    mouse.pan += delta;
                    mouse.bg_pan += delta;
                }
            }
        }
    }

    // Left drag in progress
    if mouse.left_down {
        let current_drag = mouse.left_drag.clone();
        match current_drag {
            LeftDragKind::MoveOrigin { last_logical } => {
                if resp.dragged() {
                    let (dx, dy) = (lx - last_logical.0, ly - last_logical.1);
                    mouse.pan += Vec2::new(dx as f32 * scale, dy as f32 * scale);
                    mouse.left_drag = LeftDragKind::MoveOrigin {
                        last_logical: (lx, ly),
                    };
                }
            }
            LeftDragKind::MovePoints { last_logical } => {
                if resp.dragged() {
                    let (dx, dy) = (lx - last_logical.0, ly - last_logical.1);
                    if !mouse.selected.is_empty() {
                        drawing.move_points(&mouse.selected, dx, dy);
                    } else if let Some(id) = mouse.mouse_down_id {
                        drawing.move_point(id, dx, dy);
                    }
                    mouse.left_drag = LeftDragKind::MovePoints {
                        last_logical: (lx, ly),
                    };
                    mouse.undo_desc = "Move point(s)".into();
                }
            }
            LeftDragKind::Marquee { start, .. } => {
                if let Some(pos) = pointer {
                    mouse.left_drag = LeftDragKind::Marquee {
                        start,
                        current: pos,
                    };
                    mouse.marquee = Some((start, pos));
                }
            }
            LeftDragKind::TransformHandle { corner } => {
                if resp.dragged() {
                    if let Some(ref mut q) = mouse.transform_quad {
                        q[corner] = (lx, ly);
                        let quad_copy = *q;
                        drawing.apply_bilinear(quad_copy);
                        mouse.undo_desc = "Bilinear/Scale transform".into();
                    }
                }
            }
            _ => {}
        }
    }

    // Left button down
    if resp.ctx.input(|i| i.pointer.primary_pressed()) && resp.hovered() {
        mouse.left_down = true;
        mouse.mouse_down_id = mouse.pointed_id;
        let shift = resp.ctx.input(|i| i.modifiers.shift);
        let ctrl = resp.ctx.input(|i| i.modifiers.ctrl);
        let smode = CanvasMouseState::select_mode(shift, ctrl);

        if mouse.hover_origin && mouse.pointed_id.is_none() {
            mouse.left_drag = LeftDragKind::MoveOrigin {
                last_logical: (lx, ly),
            };
        } else {
            match mouse.tool {
                Tool::Select | Tool::Delete => {
                    if let Some(id) = mouse.pointed_id {
                        mouse.apply_point_selection(id, smode);
                        mouse.left_drag = LeftDragKind::MovePoints {
                            last_logical: (lx, ly),
                        };
                    } else if let Some(pos) = pointer {
                        mouse.left_drag = LeftDragKind::Marquee {
                            start: pos,
                            current: pos,
                        };
                        if smode == SelectMode::New {
                            mouse.clear_selection();
                        }
                    }
                }
                Tool::ScaleRotate | Tool::Bilinear => {
                    mouse.init_transform_quad(drawing);
                    if let Some(q) = mouse.transform_quad {
                        let mut best = None;
                        for (i, &(cx, cy)) in q.iter().enumerate() {
                            let d = (cx - lx).hypot(cy - ly);
                            if d < hr * 2.5 && best.map_or(true, |(_, bd)| d < bd) {
                                best = Some((i, d));
                            }
                        }
                        if let Some((corner, _)) = best {
                            drawing.push_history_desc("Bilinear transform");
                            mouse.left_drag = LeftDragKind::TransformHandle { corner };
                        }
                    }
                }
                _ => {
                    mouse.left_drag = LeftDragKind::PlaceCommand;
                }
            }
        }
    }

    // Left button released
    if resp.ctx.input(|i| i.pointer.primary_released()) && mouse.left_down {
        let down_id = mouse.mouse_down_id;
        let pointed = mouse.pointed_id;

        match mouse.tool {
            Tool::Delete => {
                if down_id.is_some() && down_id == pointed {
                    if let Some(id) = down_id {
                        drawing.push_history_desc("Delete point");
                        drawing.delete_point(id);
                        mouse.clear_selection();
                        out.cmd_changed = true;
                    }
                }
            }
            Tool::Select => {
                if let LeftDragKind::Marquee { start, current } = mouse.left_drag {
                    let shift = resp.ctx.input(|i| i.modifiers.shift);
                    let ctrl = resp.ctx.input(|i| i.modifiers.ctrl);
                    let smode = CanvasMouseState::select_mode(shift, ctrl);
                    mouse.select_points_in_rect(drawing, origin, scale, start, current, smode);
                } else if matches!(mouse.left_drag, LeftDragKind::MovePoints { .. })
                    && !mouse.undo_desc.is_empty()
                {
                    drawing.push_history_desc(&mouse.undo_desc);
                    out.cmd_changed = true;
                }
            }
            Tool::MoveTo => {
                drawing.push_history_desc("Add M");
                drawing.cmd_m(lx, ly);
                out.cmd_changed = true;
            }
            Tool::MoveNoClose => {
                drawing.push_history_desc("Add N");
                drawing.cmd_n(lx, ly);
                out.cmd_changed = true;
            }
            Tool::LineTo => {
                drawing.push_history_desc("Add L");
                if drawing.commands.is_empty() {
                    drawing.cmd_m(lx, ly);
                } else {
                    drawing.cmd_l(lx, ly);
                }
                out.cmd_changed = true;
            }
            Tool::Bezier => {
                if settings.auto_bezier {
                    drawing.push_history_desc("Add B");
                    if drawing.commands.is_empty() {
                        drawing.cmd_m(lx, ly);
                    } else {
                        let _ = drawing.cmd_b_auto(lx, ly, settings.bezier_smoothness);
                    }
                    out.cmd_changed = true;
                } else {
                    mouse.temp_points.push((lx, ly));
                    if mouse.temp_points.len() == 3 {
                        let (c1x, c1y) = mouse.temp_points[0];
                        let (c2x, c2y) = mouse.temp_points[1];
                        let (ex, ey) = mouse.temp_points[2];
                        drawing.push_history_desc("Add B");
                        if drawing.commands.is_empty() {
                            drawing.cmd_m(c1x, c1y);
                        }
                        drawing.cmd_b(c1x, c1y, c2x, c2y, ex, ey);
                        mouse.temp_points.clear();
                        out.cmd_changed = true;
                    }
                }
            }
            Tool::Spline => {
                mouse.temp_points.push((lx, ly));
            }
            Tool::ExtendSpline => {
                drawing.push_history_desc("Add P");
                drawing.cmd_p(lx, ly);
                out.cmd_changed = true;
            }
            Tool::Close => {
                drawing.push_history_desc("Add C");
                drawing.cmd_c();
                out.cmd_changed = true;
            }
            Tool::ColorPick => {
                out.request_color_dialog = true;
                out.contour_for_color = drawing.contour_count().saturating_sub(1);
            }
            Tool::ScaleRotate | Tool::Bilinear => {
                if !mouse.undo_desc.is_empty() {
                    drawing.push_history_desc(&mouse.undo_desc);
                    out.cmd_changed = true;
                }
            }
            _ => {}
        }

        mouse.left_down = false;
        mouse.left_drag = LeftDragKind::None;
        mouse.mouse_down_id = None;
        mouse.marquee = None;
        mouse.undo_desc.clear();
    }

    // Double-click in Coloring or Select mode
    if resp.double_clicked() && matches!(mouse.tool, Tool::ColorPick | Tool::Select) {
        out.request_color_dialog = true;
        out.contour_for_color = drawing.contour_count().saturating_sub(1);
    }

    // Finish Spline with Enter or secondary click
    if mouse.tool == Tool::Spline && !mouse.temp_points.is_empty() {
        let finish = resp.ctx.input(|i| i.key_pressed(egui::Key::Enter));
        if finish {
            drawing.push_history_desc("Add Spline");
            if drawing.commands.is_empty() {
                if let Some(&(x, y)) = mouse.temp_points.first() {
                    drawing.cmd_m(x, y);
                }
            }
            if mouse.temp_points.len() >= 2 {
                let pts = mouse.temp_points.clone();
                drawing.cmd_s(&pts);
                drawing.cmd_c();
            }
            mouse.temp_points.clear();
            out.cmd_changed = true;
        }
    }

    // Context Menu
    let pointed_id = mouse.pointed_id;
    let has_sel = !mouse.selected.is_empty();
    resp.context_menu(|ui| {
        if ui.button(t(lang, "ctx_line_to_bez")).clicked() {
            drawing.convert_lines_to_bezier(settings.bezier_smoothness);
            out.cmd_changed = true;
            ui.close_menu();
        }
        if ui.button(t(lang, "ctx_bez_to_line")).clicked() {
            drawing.convert_bezier_to_lines();
            out.cmd_changed = true;
            ui.close_menu();
        }
        if let Some(id) = pointed_id {
            if ui.button(t(lang, "ctx_c1_continuity")).clicked() {
                drawing.push_history_desc("Toggle C1 Continuity");
                drawing.toggle_c1_continuity(id);
                out.cmd_changed = true;
                ui.close_menu();
            }
        }
        ui.separator();
        if ui.button(t(lang, "ctx_move_00_here")).clicked() {
            drawing.push_history_desc("Move [0,0]");
            drawing.move_points(&drawing.points.keys().copied().collect::<Vec<_>>(), -lx, -ly);
            out.cmd_changed = true;
            ui.close_menu();
        }
        if ui.button(t(lang, "action_recenter")).clicked() {
            mouse.pan = Vec2::ZERO;
            ui.close_menu();
        }
        ui.separator();
        if has_sel && ui.button(t(lang, "ctx_delete_point")).clicked() {
            out.request_delete_sel = true;
            ui.close_menu();
        }
        if ui.button(t(lang, "action_clear_selection")).clicked() {
            mouse.clear_selection();
            ui.close_menu();
        }
    });

    out
}
