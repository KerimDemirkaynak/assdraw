//! egui central panel — wires `CanvasMouseState` to `Drawing` (behaviour of canvas.cpp:
//! OnMouseLeftDown/Up/Move, OnMouseRight*, wheel/keys, transform modes, coloring).

use crate::app::AssDrawApp;
use crate::canvas::fill::{build_filled_meshes, extract_contours};
use crate::canvas::mouse::{
    CanvasMouseState, DragTarget, LeftDragKind, MouseFrameResult, SelectMode, HANDLE_HIT_RADIUS,
};
use crate::canvas::render;
use crate::engine::{DrawCmd, Drawing, PointKind};
use crate::i18n::t;
use crate::settings::AppSettings;
use crate::ui::Tool;
use egui::{Pos2, Rect, Sense, Vec2};

pub fn canvas(app: &mut AssDrawApp, ctx: &egui::Context) {
    // A tool change cancels gestures, clears the selection and rebuilds transform handles
    // (the original's SetDrawMode).
    if app.tool != app.mouse.tool {
        app.mouse.temp_points.clear();
        app.mouse.left_drag = LeftDragKind::None;
        app.mouse.left_down = false;
        app.mouse.undo_desc.clear();
        app.mouse.marquee = None;
        app.mouse.coloring_target = None;
        app.mouse.mouse_down_id = None;
        app.mouse.clear_selection();
        app.mouse.reset_transform();
    }
    app.mouse.tool = app.tool;

    // Drag-and-drop of image files as background (copy first: loading a texture while the input
    // lock is held would deadlock).
    let dropped = ctx.input(|i| i.raw.dropped_files.clone());
    if let Some(file) = dropped.into_iter().next() {
        if let Some(path) = &file.path {
            app.load_bg_from_path(path, ctx);
        } else if let Some(bytes) = &file.bytes {
            app.load_bg_from_bytes(bytes, ctx);
        }
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
            let rect = resp.rect;
            app.canvas_rect = rect;

            let result = process_input(&mut app.mouse, &mut app.drawing, &app.settings, &resp, rect);

            app.last_cursor = (f64::from(result.status_coords.0), f64::from(result.status_coords.1));
            app.status_cmd = result.status_cmd.clone();
            if let Some(tool) = result.set_tool {
                app.tool = tool;
            }
            if let Some(key) = result.message {
                let lang = app.settings.language;
                app.set_status(t(lang, key));
            }
            if let Some(idx) = result.request_color_dialog {
                app.open_color_dialog(idx);
            }
            if result.cmd_changed {
                app.sync_cmd();
            }
            app.selected = app.mouse.selected.clone();

            // Geometry may have changed (zoom / pan) during input processing.
            let origin = app.mouse.origin(rect);
            let scale = app.mouse.view_zoom;
            let to_screen = move |lx: f64, ly: f64| -> Pos2 {
                Pos2::new(origin.x + lx as f32 * scale, origin.y + ly as f32 * scale)
            };

            painter.rect_filled(rect, 0.0, app.settings.canvas_bg.to_egui());

            // Tracing background image (position is relative to the origin, see `pan_by`).
            if let Some(tex) = &app.bg_image {
                let size = tex.size_vec2() * app.bg_scale * app.mouse.bg_zoom;
                let img_rect = Rect::from_center_size(origin + app.mouse.bg_pan, size);
                painter.image(
                    tex.id(),
                    img_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    egui::Color32::from_white_alpha(app.bg_alpha),
                );
            }

            if app.settings.show_grid && !app.mouse.preview_mode {
                render::draw_grid(&painter, rect, origin, scale);
            }

            // Filled sub-shapes (non-zero winding, holes stay holes).
            let fill_color = app.settings.shape_fill.to_egui();
            let meshes = build_filled_meshes(&app.drawing, &to_screen, fill_color, app.settings.fill_opacity);
            for fs in meshes {
                painter.add(egui::Shape::mesh(fs.mesh));
            }

            // Polylines of every contour in screen space.
            let tol = (0.25 / f64::from(scale)).min(0.35);
            let polylines: Vec<Vec<Pos2>> = extract_contours(&app.drawing, tol)
                .iter()
                .map(|c| c.points.iter().map(|&(x, y)| to_screen(x, y)).collect())
                .collect();

            // ASS border preview and coloring-target highlight.
            let global_style = app.settings.shape_style();
            for (outer, members) in app.drawing.subshape_groups() {
                let st = app.drawing.shape_styles.get(&outer).copied().unwrap_or(global_style);
                if st.outline_enabled && st.outline_width > 0.0 {
                    let col = egui::Color32::from_rgba_unmultiplied(
                        st.outline.r,
                        st.outline.g,
                        st.outline.b,
                        st.outline_opacity,
                    );
                    for &m in &members {
                        if let Some(pl) = polylines.get(m) {
                            render::draw_style_outline(&painter, pl, st.outline_width as f32 * scale, col);
                        }
                    }
                }
                if app.tool == Tool::ColorPick && app.mouse.coloring_target == Some(outer) {
                    let sel = app.settings.point_selected.to_egui();
                    for &m in &members {
                        if let Some(pl) = polylines.get(m) {
                            render::draw_style_outline(&painter, pl, 2.0, sel);
                        }
                    }
                }
            }

            if !app.mouse.preview_mode {
                render::draw_editor_outline(&painter, &polylines, &app.settings);
                render::draw_origin(
                    &painter,
                    origin,
                    app.settings.origin_cross_size as f32 * scale,
                    app.settings.origin_color.to_egui(),
                );
                if app.settings.show_points {
                    render::draw_points(&painter, &app.drawing, &app.mouse, origin, scale, &app.settings, rect);
                }
                if app.settings.show_rulers {
                    render::draw_rulers(&painter, rect, origin, scale, app.settings.ruler_h_color.to_egui());
                }
            }
        });
}

fn logical(pos: Pos2, origin: Pos2, scale: f32) -> (f64, f64) {
    (
        f64::from((pos.x - origin.x) / scale),
        f64::from((pos.y - origin.y) / scale),
    )
}

fn snap(v: (f64, f64)) -> (f64, f64) {
    (v.0.round(), v.1.round())
}

fn cmd_letter(c: &DrawCmd) -> char {
    match c {
        DrawCmd::M { .. } => 'M',
        DrawCmd::N { .. } => 'N',
        DrawCmd::L { .. } => 'L',
        DrawCmd::B { .. } => 'B',
        DrawCmd::S { .. } => 'S',
        DrawCmd::P { .. } => 'P',
        DrawCmd::C => 'C',
    }
}

fn is_transform_tool(tool: Tool) -> bool {
    matches!(tool, Tool::ScaleRotate | Tool::Bilinear)
}

/// Record the undo snapshot once per gesture, before the first modification.
fn begin_edit(mouse: &mut CanvasMouseState, drawing: &mut Drawing, desc: &str) {
    if mouse.undo_desc.is_empty() {
        drawing.push_history_desc(desc);
        mouse.undo_desc = desc.to_string();
    }
}

fn process_input(
    mouse: &mut CanvasMouseState,
    drawing: &mut Drawing,
    settings: &AppSettings,
    resp: &egui::Response,
    rect: Rect,
) -> MouseFrameResult {
    let mut out = MouseFrameResult::default();
    let ctx = resp.ctx.clone();
    let tool = mouse.tool;
    let (mods, escape, enter, tab, primary_pressed, primary_down, primary_released, scroll, press_origin, delta) =
        ctx.input(|i| {
            (
                i.modifiers,
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Tab),
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
                i.pointer.primary_released(),
                i.smooth_scroll_delta.y,
                i.pointer.press_origin(),
                i.pointer.delta(),
            )
        });
    let pointer = resp.hover_pos().or(resp.interact_pointer_pos());

    // Escape: leave preview, or cancel a multi-click gesture.
    if escape {
        if mouse.preview_mode {
            mouse.preview_mode = false;
        }
        mouse.temp_points.clear();
    }

    // Mouse wheel zoom (works in preview mode too).
    if scroll != 0.0 && resp.hovered() {
        let amount = scroll / 50.0 / if mods.ctrl { 10.0 } else { 1.0 };
        mouse.zoom_by(rect, amount, pointer);
    }

    let mut origin = mouse.origin(rect);
    let mut scale = mouse.view_zoom;

    // Right / middle drag pans, or rotates in Scale/Rotate mode.
    let rotating = tool == Tool::ScaleRotate && mouse.transform_quad.is_some() && !mouse.preview_mode;
    if rotating && resp.drag_started_by(egui::PointerButton::Secondary) {
        if let (Some(q), Some(start)) = (mouse.transform_quad, press_origin) {
            let cx = (q[0].0 + q[2].0) * 0.5;
            let cy = (q[0].1 + q[2].1) * 0.5;
            let radius = (q[0].0 - cx).hypot(q[0].1 - cy).max(1e-3);
            mouse.transform_start_quad = q;
            mouse.rotate_start_logical = Some(logical(start, origin, scale));
            mouse.rotate_center_radius = Some(((cx, cy), radius));
        }
    }
    if rotating && resp.dragged_by(egui::PointerButton::Secondary) {
        if let (Some(start_m), Some(((cx, cy), radius)), Some(pos), Some((entry_pts, entry_rect))) = (
            mouse.rotate_start_logical,
            mouse.rotate_center_radius,
            pointer,
            mouse.transform_entry.take(),
        ) {
            let (lx, ly) = logical(pos, origin, scale);
            begin_edit(mouse, drawing, "Rotate");
            let backup = mouse.transform_start_quad;
            let delta_angle = (ly - cy).atan2(lx - cx) - (start_m.1 - cy).atan2(start_m.0 - cx);
            let mut q = backup;
            for i in 0..4 {
                let a = (backup[i].1 - cy).atan2(backup[i].0 - cx) + delta_angle;
                q[i] = ((cx + radius * a.cos()).round(), (cy + radius * a.sin()).round());
            }
            mouse.transform_quad = Some(q);
            drawing.apply_bilinear_from(&entry_pts, entry_rect, q);
            mouse.transform_entry = Some((entry_pts, entry_rect));
            out.cmd_changed = true;
        }
    } else if resp.dragged_by(egui::PointerButton::Secondary) || resp.dragged_by(egui::PointerButton::Middle) {
        mouse.pan_by(delta);
    }
    if resp.drag_stopped_by(egui::PointerButton::Secondary) && !mouse.undo_desc.is_empty() && !mouse.left_down {
        mouse.undo_desc.clear();
        mouse.rotate_start_logical = None;
        mouse.rotate_center_radius = None;
        out.cmd_changed = true;
    }
    origin = mouse.origin(rect);
    scale = mouse.view_zoom;

    let (lx, ly) = pointer.map(|p| logical(p, origin, scale)).unwrap_or((0.0, 0.0));
    let (sx, sy) = snap((lx, ly));
    out.status_coords = (sx as i32, sy as i32);

    if mouse.preview_mode {
        return out;
    }

    // Transform tools need their handles (rebuilt from the shape when missing).
    if is_transform_tool(tool) && mouse.transform_quad.is_none() && !mouse.left_down {
        mouse.init_transform(drawing, scale);
    }

    // Hover (not while a gesture is running, not in transform modes).
    if !mouse.left_down && !is_transform_tool(tool) {
        mouse.pointed_id = pointer
            .filter(|_| resp.hovered())
            .and_then(|p| CanvasMouseState::find_point_at(drawing, p, origin, scale));
    }
    if let Some(id) = mouse.pointed_id {
        if let Some(i) = drawing.cmd_index_of_point(id) {
            out.status_cmd = drawing.cmd_text(i).to_uppercase();
        }
    }

    // ------------------------------------------------------------------ left button down
    if primary_pressed && resp.hovered() {
        if let Some(pos) = pointer {
            on_left_down(mouse, drawing, &mods, pos, (lx, ly), origin, scale, &mut out);
        }
    }

    // ------------------------------------------------------------------ left drag
    if mouse.left_down && primary_down {
        on_left_drag(mouse, drawing, &mods, pointer, (lx, ly), origin, scale);
    }

    // ------------------------------------------------------------------ left button released
    if mouse.left_down && primary_released {
        on_left_up(mouse, drawing, pointer, origin, scale, &mut out);
    }

    // Double click in Coloring mode: choose the fill colour of the sub-shape.
    if tool == Tool::ColorPick && resp.double_clicked() {
        mouse.coloring_target = drawing.subshape_at(lx, ly);
        match mouse.coloring_target {
            Some(idx) => out.request_color_dialog = Some(idx),
            None => out.message = Some("msg_color_pick_hint"),
        }
    }

    // Spline tool: Enter or right click finishes the spline.
    let mut consume_secondary = false;
    if tool == Tool::Spline && !mouse.temp_points.is_empty() {
        let finish = enter || resp.secondary_clicked();
        if finish {
            if mouse.temp_points.len() >= 3 {
                drawing.push_history_desc("Add a new S");
                if drawing.commands.is_empty() {
                    let (x, y) = mouse.temp_points[0];
                    drawing.cmd_m(x, y);
                }
                let pts = mouse.temp_points.clone();
                drawing.cmd_s(&pts);
                out.cmd_changed = true;
            }
            mouse.temp_points.clear();
            consume_secondary = resp.secondary_clicked();
        }
    }

    // Tab: jump the pointer to the nearest point, or on to the next point of the drawing.
    if tab && resp.hovered() && !mouse.left_down && !is_transform_tool(tool) && !drawing.points.is_empty() {
        let order = CanvasMouseState::tab_order(drawing);
        let target = match mouse.pointed_id.and_then(|id| order.iter().position(|&x| x == id)) {
            Some(i) => order.get((i + 1) % order.len()).copied(),
            None => pointer.and_then(|pos| {
                order
                    .iter()
                    .filter_map(|id| drawing.points.get(id).map(|p| (*id, CanvasMouseState::point_screen(p, origin, scale))))
                    .min_by(|a, b| a.1.distance(pos).total_cmp(&b.1.distance(pos)))
                    .map(|(id, _)| id)
            }),
        };
        if let Some(p) = target.and_then(|id| drawing.points.get(&id)) {
            let screen = CanvasMouseState::point_screen(p, origin, scale);
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorPosition(screen));
        }
    }

    if resp.secondary_clicked() && !consume_secondary {
        mouse.ctx_logical = (sx, sy);
        mouse.ctx_point = mouse.pointed_id;
    }

    if !consume_secondary {
        context_menu(mouse, drawing, settings, resp, &mut out);
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn on_left_down(
    mouse: &mut CanvasMouseState,
    drawing: &mut Drawing,
    mods: &egui::Modifiers,
    pos: Pos2,
    logical_pos: (f64, f64),
    origin: Pos2,
    scale: f32,
    out: &mut MouseFrameResult,
) {
    let tool = mouse.tool;
    mouse.left_down = true;
    mouse.undo_desc.clear();
    mouse.mouse_down_id = None;
    mouse.left_drag = LeftDragKind::None;
    let (px, py) = snap(logical_pos);

    // Coloring: a click only selects the sub-shape.
    if tool == Tool::ColorPick {
        mouse.coloring_target = drawing.subshape_at(logical_pos.0, logical_pos.1);
        if mouse.coloring_target.is_none() {
            out.message = Some("msg_color_pick_hint");
        }
        return;
    }

    // Scale/Rotate and Bilinear: pick a corner or an edge handle.
    if is_transform_tool(tool) {
        if let Some(q) = mouse.transform_quad {
            let screen: Vec<Pos2> = q
                .iter()
                .map(|&(x, y)| Pos2::new(origin.x + x as f32 * scale, origin.y + y as f32 * scale))
                .collect();
            let mut handle: Option<(usize, Option<usize>)> = None;
            let mut best = f32::MAX;
            for (i, s) in screen.iter().enumerate() {
                let d = s.distance(pos);
                if d <= HANDLE_HIT_RADIUS && d < best {
                    best = d;
                    handle = Some((i, None));
                }
            }
            if handle.is_none() {
                for i in 0..4 {
                    let j = (i + 1) % 4;
                    let (a, b) = (screen[i], screen[j]);
                    let ab = b - a;
                    let len2 = ab.length_sq();
                    if len2 < 1e-6 {
                        continue;
                    }
                    let tpar = ((pos - a).dot(ab)) / len2;
                    // Only the middle third of an edge grabs it (corners keep priority).
                    if tpar <= 1.0 / 3.0 || tpar >= 2.0 / 3.0 {
                        continue;
                    }
                    let d = (a + ab * tpar).distance(pos);
                    if d <= HANDLE_HIT_RADIUS && d < best {
                        best = d;
                        handle = Some((i, Some(j)));
                    }
                }
            }
            if let Some((corner, partner)) = handle {
                mouse.transform_start_quad = q;
                mouse.left_drag = LeftDragKind::TransformHandle {
                    corner,
                    edge_partner: partner,
                    start_logical: logical_pos,
                    last_logical: logical_pos,
                };
            }
        }
        return;
    }

    // Hit test again: a click may arrive before any motion event.
    let pointed = CanvasMouseState::find_point_at(drawing, pos, origin, scale);
    mouse.pointed_id = pointed;
    let on_control = pointed
        .and_then(|id| drawing.points.get(&id))
        .is_some_and(|p| p.kind == PointKind::Control);

    // Draw tools create a command at the click; clicking an existing point inserts it after that
    // point's command, clicking a control point drags the control instead.
    let kind = match tool {
        Tool::MoveTo => Some('m'),
        Tool::MoveNoClose => Some('n'),
        Tool::LineTo => Some('l'),
        Tool::Bezier => Some('b'),
        _ => None,
    };
    if let Some(kind) = kind {
        if !on_control {
            let desc = format!("Add a new {}", kind.to_ascii_uppercase());
            begin_edit(mouse, drawing, &desc);
            let after = pointed.and_then(|id| drawing.cmd_index_of_point(id));
            let (idx, pid) = drawing.place_cmd(kind, px, py, after);
            mouse.mouse_down_id = Some(pid);
            mouse.pointed_id = Some(pid);
            mouse.selected = vec![pid];
            mouse.left_drag = LeftDragKind::PlaceCommand { cmd_idx: idx, point: pid };
            out.cmd_changed = true;
            return;
        }
    }

    // Spline tool collects points; extend / close tools act on the last spline.
    match tool {
        Tool::Spline => {
            mouse.temp_points.push((px, py));
            return;
        }
        Tool::ExtendSpline => {
            drawing.push_history_desc("Extend spline");
            if drawing.extend_last_spline(px, py) {
                out.cmd_changed = true;
            } else {
                drawing.undo_discard();
                out.message = Some("msg_need_spline");
            }
            return;
        }
        Tool::Close => {
            drawing.push_history_desc("Close spline");
            if drawing.close_last_spline() {
                out.cmd_changed = true;
            } else {
                drawing.undo_discard();
                out.message = Some("msg_need_spline");
            }
            return;
        }
        _ => {}
    }

    // Selection (Ctrl adds, Alt removes). Clicking an already selected point keeps the selection
    // so the whole group can be dragged.
    let smode = CanvasMouseState::select_mode(mods.ctrl || mods.command, mods.alt);
    mouse.mouse_down_id = pointed;
    if let Some(id) = pointed {
        if !mouse.selected.contains(&id) {
            mouse.apply_point_selection(id, smode);
        }
        if tool != Tool::Delete {
            mouse.left_drag = LeftDragKind::MovePoints;
        }
    } else {
        if smode == SelectMode::New {
            mouse.clear_selection();
        }
        if px == 0.0 && py == 0.0 {
            mouse.left_drag = LeftDragKind::MoveOrigin;
        } else if tool != Tool::Delete && tool != Tool::Pan {
            mouse.left_drag = LeftDragKind::Marquee { start: pos, current: pos };
        }
    }
}

fn on_left_drag(
    mouse: &mut CanvasMouseState,
    drawing: &mut Drawing,
    mods: &egui::Modifiers,
    pointer: Option<Pos2>,
    logical_pos: (f64, f64),
    origin: Pos2,
    scale: f32,
) {
    let Some(pos) = pointer else { return };
    let (wx, wy) = snap(logical_pos);
    match mouse.left_drag.clone() {
        LeftDragKind::PlaceCommand { cmd_idx, point } => {
            drawing.set_point_pos(point, wx, wy);
            if matches!(drawing.commands.get(cmd_idx), Some(DrawCmd::B { .. })) {
                drawing.reinit_bezier_controls(cmd_idx);
            }
        }
        LeftDragKind::MovePoints => {
            let Some(down) = mouse.mouse_down_id else { return };
            if !mouse.selected.contains(&down) {
                return;
            }
            let Some(p) = drawing.points.get(&down).copied() else { return };
            if (p.x, p.y) == (wx, wy) {
                return;
            }
            let desc = if p.kind == PointKind::Main { "Drag point" } else { "Drag control point" };
            begin_edit(mouse, drawing, desc);
            if mouse.tool == Tool::Select {
                let sel = mouse.selected.clone();
                drawing.move_points(&sel, wx - p.x, wy - p.y);
            } else {
                drawing.set_point_pos(down, wx, wy);
            }
        }
        LeftDragKind::MoveOrigin => {
            if wx != 0.0 || wy != 0.0 {
                begin_edit(mouse, drawing, "Move origin");
                for p in drawing.points.values_mut() {
                    p.x -= wx;
                    p.y -= wy;
                }
                // The origin follows the cursor; shape and background stay put on screen.
                let shift = Vec2::new(wx as f32 * scale, wy as f32 * scale);
                mouse.pan += shift;
                mouse.bg_pan -= shift;
            }
        }
        LeftDragKind::Marquee { start, .. } => {
            mouse.left_drag = LeftDragKind::Marquee { start, current: pos };
            mouse.marquee = Some((start, pos));
            let mode = CanvasMouseState::select_mode(mods.ctrl || mods.command, mods.alt);
            mouse.select_points_in_rect(drawing, origin, scale, start, pos, mode);
        }
        LeftDragKind::TransformHandle { .. } => {
            let desc = if mouse.tool == Tool::ScaleRotate { "Scale" } else { "Bilinear transform" };
            begin_edit(mouse, drawing, desc);
            drag_transform(mouse, drawing, scale, logical_pos, mods.shift);
        }
        LeftDragKind::None => {}
    }
}

fn on_left_up(
    mouse: &mut CanvasMouseState,
    drawing: &mut Drawing,
    pointer: Option<Pos2>,
    origin: Pos2,
    scale: f32,
    out: &mut MouseFrameResult,
) {
    // Delete: pressing and releasing on the same point removes its command.
    if mouse.tool == Tool::Delete {
        let released_on = pointer.and_then(|p| CanvasMouseState::find_point_at(drawing, p, origin, scale));
        if let (Some(down), Some(up)) = (mouse.mouse_down_id, released_on) {
            if down == up {
                match drawing.cmd_index_of_point(down) {
                    Some(0) if drawing.commands.len() > 1 => out.message = Some("msg_delete_last"),
                    Some(idx) => {
                        let desc = format!("Delete a {}", cmd_letter(&drawing.commands[idx]));
                        drawing.push_history_desc(&desc);
                        drawing.delete_command_of_point(down);
                        mouse.clear_selection();
                        mouse.pointed_id = None;
                        out.cmd_changed = true;
                    }
                    None => {}
                }
            }
        }
    }
    if !mouse.undo_desc.is_empty() {
        out.cmd_changed = true;
    }
    mouse.left_down = false;
    mouse.left_drag = LeftDragKind::None;
    mouse.mouse_down_id = None;
    mouse.marquee = None;
    mouse.undo_desc.clear();
}

/// Scale/rotate (corner + edge handles, Shift keeps the aspect ratio) and bilinear deformation.
/// The shape is always mapped from the points it had when the tool was entered.
fn drag_transform(
    mouse: &mut CanvasMouseState,
    drawing: &mut Drawing,
    scale: f32,
    cur: (f64, f64),
    shift: bool,
) {
    let LeftDragKind::TransformHandle { corner, edge_partner, start_logical, last_logical } =
        mouse.left_drag.clone()
    else {
        return;
    };
    let Some(mut q) = mouse.transform_quad else { return };
    let Some((entry_pts, entry_rect)) = mouse.transform_entry.take() else { return };
    let backup = mouse.transform_start_quad;
    let th = 10.0 / f64::from(scale);
    let diff = (cur.0 - start_logical.0, cur.1 - start_logical.1);

    if mouse.tool == Tool::ScaleRotate {
        if let Some(partner) = edge_partner {
            // An edge moves two vertices in sync.
            let toupd = [corner, partner];
            let adjacent = [(partner + 2) % 4, (partner + 1) % 4];
            let v1 = (backup[toupd[0]].0 + diff.0, backup[toupd[0]].1 + diff.1);
            let v2 = (backup[toupd[1]].0 + diff.0, backup[toupd[1]].1 + diff.1);
            for i in 0..2 {
                let v = toupd[i];
                let a = adjacent[i];
                if let Some((nx, ny)) = line_intersect(q[a].0, q[a].1, backup[v].0, backup[v].1, v1.0, v1.1, v2.0, v2.1) {
                    if (nx - backup[a].0).abs() > th || (ny - backup[a].1).abs() > th {
                        q[v] = (nx.round(), ny.round());
                    }
                }
            }
        } else {
            let opp = (corner + 2) % 4;
            let adj = [(corner + 3) % 4, (corner + 1) % 4];
            let newpoint = (backup[corner].0 + diff.0, backup[corner].1 + diff.1);
            for &a in &adj {
                if let Some((nx, ny)) = line_intersect(
                    backup[opp].0, backup[opp].1, backup[a].0, backup[a].1,
                    newpoint.0, newpoint.1, backup[a].0 + diff.0, backup[a].1 + diff.1,
                ) {
                    if (nx - backup[opp].0).abs() > th || (ny - backup[opp].1).abs() > th {
                        q[a] = (nx, ny);
                    }
                }
            }
            q[corner] = (q[adj[0]].0 + q[adj[1]].0 - backup[opp].0, q[adj[0]].1 + q[adj[1]].1 - backup[opp].1);

            if shift {
                // Keep the aspect ratio: port of the original guide-point construction.
                let mut guide = [(0.0, 0.0); 4];
                guide[opp] = backup[opp];
                guide[adj[0]] = backup[adj[0]];
                guide[adj[1]] = backup[adj[1]];
                for &a in &adj {
                    let g = guide[a];
                    let go = guide[opp];
                    if (q[a].0 < go.0 && go.0 < g.0)
                        || (q[a].0 > go.0 && go.0 > g.0)
                        || (q[a].1 < go.1 && go.1 < g.1)
                        || (q[a].1 > go.1 && go.1 > g.1)
                    {
                        guide[a] = (go.0 - (g.0 - go.0), go.1 - (g.1 - go.1));
                    }
                }
                guide[corner] = (
                    guide[adj[0]].0 + guide[adj[1]].0 - guide[opp].0,
                    guide[adj[0]].1 + guide[adj[1]].1 - guide[opp].1,
                );
                let mut inter = [(0.0, 0.0); 2];
                let mut d = [(0.0, 0.0); 2];
                for i in 0..2 {
                    if let Some((x, y)) = line_intersect(
                        guide[opp].0, guide[opp].1, guide[corner].0, guide[corner].1,
                        q[corner].0, q[corner].1, q[adj[i]].0, q[adj[i]].1,
                    ) {
                        inter[i] = (x, y);
                        d[i] = (x - guide[opp].0, y - guide[opp].1);
                    }
                }
                for i in 0..2 {
                    let j = 1 - i;
                    if d[i].0.abs() > d[j].0.abs() || d[i].1.abs() > d[j].1.abs() {
                        q[corner] = inter[i];
                        q[adj[j]] = (
                            q[adj[i]].0 + inter[i].0 - guide[opp].0,
                            q[adj[i]].1 + inter[i].1 - guide[opp].1,
                        );
                    }
                }
            }
        }
    } else if let Some(partner) = edge_partner {
        // Bilinear: an edge moves both of its vertices by the mouse movement.
        let dx = cur.0 - last_logical.0;
        let dy = cur.1 - last_logical.1;
        q[corner] = (q[corner].0 + dx, q[corner].1 + dy);
        q[partner] = (q[partner].0 + dx, q[partner].1 + dy);
    } else {
        // Bilinear: a corner follows the mouse.
        q[corner] = (cur.0.round(), cur.1.round());
    }

    mouse.transform_quad = Some(q);
    mouse.left_drag = LeftDragKind::TransformHandle {
        corner,
        edge_partner,
        start_logical,
        last_logical: cur,
    };
    drawing.apply_bilinear_from(&entry_pts, entry_rect, q);
    mouse.transform_entry = Some((entry_pts, entry_rect));
}

fn context_menu(
    mouse: &mut CanvasMouseState,
    drawing: &mut Drawing,
    settings: &AppSettings,
    resp: &egui::Response,
    out: &mut MouseFrameResult,
) {
    let lang = settings.language;
    let current_tool = mouse.tool;
    let point = mouse.ctx_point;
    let ctx_logical = mouse.ctx_logical;
    let has_sel = !mouse.selected.is_empty();
    resp.context_menu(|ui| {
        if let Some(id) = point.filter(|id| drawing.points.contains_key(id)) {
            // Menu of a point: the command text (disabled) and the conversions that apply to it.
            let idx = drawing.cmd_index_of_point(id);
            if let Some(i) = idx {
                ui.add_enabled(false, egui::Button::new(drawing.cmd_text(i)));
            }
            let is_main = drawing.points.get(&id).is_some_and(|p| p.kind == PointKind::Main);
            match idx.map(|i| &drawing.commands[i]) {
                Some(DrawCmd::L { .. }) => {
                    if ui.button(t(lang, "ctx_line_to_bez")).clicked() {
                        drawing.convert_line_to_bezier_at(id);
                        mouse.clear_selection();
                        mouse.pointed_id = None;
                        out.cmd_changed = true;
                        ui.close_menu();
                    }
                }
                Some(DrawCmd::B { .. }) if is_main => {
                    if ui.button(t(lang, "ctx_bez_to_line")).clicked() {
                        drawing.convert_bezier_to_line_at(id);
                        mouse.clear_selection();
                        mouse.pointed_id = None;
                        out.cmd_changed = true;
                        ui.close_menu();
                    }
                    if drawing.can_toggle_c1(id) {
                        let on = drawing.is_c1(id);
                        if ui.selectable_label(on, t(lang, "ctx_c1_continuity")).clicked() {
                            drawing.push_history_desc(if on { "Unset continuous" } else { "Set continuous" });
                            drawing.toggle_c1_continuity(id);
                            out.cmd_changed = true;
                            ui.close_menu();
                        }
                    }
                }
                _ => {}
            }
        } else {
            if ui.button(t(lang, "ctx_move_00_here")).clicked() {
                // The origin moves onto the spot where the menu was opened; shape and background
                // keep their place on screen.
                let (cx, cy) = ctx_logical;
                if cx != 0.0 || cy != 0.0 {
                    drawing.push_history_desc("Move origin");
                    for p in drawing.points.values_mut() {
                        p.x -= cx;
                        p.y -= cy;
                    }
                    let shift = Vec2::new(cx as f32 * mouse.view_zoom, cy as f32 * mouse.view_zoom);
                    mouse.pan += shift;
                    mouse.bg_pan -= shift;
                    mouse.reset_transform();
                    out.cmd_changed = true;
                }
                ui.close_menu();
            }
            ui.separator();
            for (tool, key) in [
                (Tool::Select, "tool_select"),
                (Tool::MoveTo, "tool_m"),
                (Tool::LineTo, "tool_l"),
                (Tool::Bezier, "tool_b"),
                (Tool::Delete, "tool_del"),
                (Tool::ScaleRotate, "tool_scale_rotate"),
                (Tool::Bilinear, "tool_bilinear"),
                (Tool::ColorPick, "rb_color"),
            ] {
                if ui.radio(current_tool == tool, t(lang, key)).clicked() {
                    out.set_tool = Some(tool);
                    ui.close_menu();
                }
            }
            if has_sel {
                ui.separator();
                if ui.button(t(lang, "action_clear_selection")).clicked() {
                    mouse.clear_selection();
                    ui.close_menu();
                }
            }
        }
    });
}

/// Intersection of the infinite lines (a,b) and (c,d) (port of `agg::calc_intersection`).
fn line_intersect(
    ax: f64, ay: f64, bx: f64, by: f64,
    cx: f64, cy: f64, dx: f64, dy: f64,
) -> Option<(f64, f64)> {
    let den = (bx - ax) * (dy - cy) - (by - ay) * (dx - cx);
    if den.abs() < 1e-12 {
        return None;
    }
    let t = ((cx - ax) * (dy - cy) - (cy - ay) * (dx - cx)) / den;
    Some((ax + t * (bx - ax), ay + t * (by - ay)))
}

#[allow(dead_code)]
fn _unused(_: DragTarget) {}
