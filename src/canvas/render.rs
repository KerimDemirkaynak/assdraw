//! Canvas painting helpers: Grid, Rulers, Origin Crosshair, Paths, Points & Handles, Selection Marquee, Transform Quad.

use crate::canvas::mouse::CanvasMouseState;
use crate::engine::{Color, DrawCmd, Drawing, PointKind};
use crate::ui::Tool;
use crate::settings::AppSettings;
use egui::{Color32, FontId, Pos2, Rect, Stroke};

pub fn draw_grid(painter: &egui::Painter, rect: Rect, origin: Pos2, scale: f32) {
    if scale < 2.5 {
        return;
    }
    let (g, m) = (Color32::from_gray(40), Color32::from_gray(65));
    let step = scale;
    let sx = ((rect.min.x - origin.x) / step).floor() * step + origin.x;
    let sy = ((rect.min.y - origin.y) / step).floor() * step + origin.y;

    let mut x = sx;
    let mut i = 0;
    while x < rect.max.x {
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(1.0_f32, if i % 5 == 0 { m } else { g }),
        );
        x += step;
        i += 1;
    }

    let mut y = sy;
    let mut j = 0;
    while y < rect.max.y {
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            Stroke::new(1.0_f32, if j % 5 == 0 { m } else { g }),
        );
        y += step;
        j += 1;
    }

    // Main Axes
    painter.line_segment(
        [Pos2::new(origin.x, rect.min.y), Pos2::new(origin.x, rect.max.y)],
        Stroke::new(1.5_f32, Color32::from_rgb(140, 60, 60)),
    );
    painter.line_segment(
        [Pos2::new(rect.min.x, origin.y), Pos2::new(rect.max.x, origin.y)],
        Stroke::new(1.5_f32, Color32::from_rgb(60, 130, 60)),
    );
}

pub fn draw_rulers(
    painter: &egui::Painter,
    rect: Rect,
    origin: Pos2,
    scale: f32,
    ruler_color: Color32,
) {
    let font = FontId::monospace(9.0);
    let text_col = Color32::from_gray(180);

    // Horizontal Ruler Bar (top 16px)
    let h_rect = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.min.y + 16.0));
    painter.rect_filled(h_rect, 0.0, Color32::from_rgba_unmultiplied(20, 20, 20, 220));
    painter.line_segment([h_rect.left_bottom(), h_rect.right_bottom()], Stroke::new(1.0_f32, ruler_color));

    // Vertical Ruler Bar (left 20px)
    let v_rect = Rect::from_min_max(rect.min, Pos2::new(rect.min.x + 20.0, rect.max.y));
    painter.rect_filled(v_rect, 0.0, Color32::from_rgba_unmultiplied(20, 20, 20, 220));
    painter.line_segment([v_rect.right_top(), v_rect.right_bottom()], Stroke::new(1.0_f32, ruler_color));

    // Corner junction
    let corner = Rect::from_min_max(rect.min, Pos2::new(rect.min.x + 20.0, rect.min.y + 16.0));
    painter.rect_filled(corner, 0.0, Color32::from_rgb(35, 35, 35));

    // Ticks on H Ruler
    let step = if scale >= 7.0 {
        5.0 * scale
    } else if scale >= 3.5 {
        10.0 * scale
    } else if scale >= 1.5 {
        25.0 * scale
    } else {
        50.0 * scale
    };
    let sx = ((rect.min.x - origin.x) / step).floor() * step + origin.x;
    let mut x = sx;
    while x < rect.max.x {
        if x >= rect.min.x + 20.0 {
            let logical_val = ((x - origin.x) / scale).round() as i64;
            painter.line_segment(
                [Pos2::new(x, rect.min.y + 10.0), Pos2::new(x, rect.min.y + 16.0)],
                Stroke::new(1.0_f32, Color32::from_gray(140)),
            );
            painter.text(
                Pos2::new(x + 2.0, rect.min.y + 2.0),
                egui::Align2::LEFT_TOP,
                format!("{logical_val}"),
                font.clone(),
                text_col,
            );
        }
        x += step;
    }

    // Ticks on V Ruler
    let sy = ((rect.min.y - origin.y) / step).floor() * step + origin.y;
    let mut y = sy;
    while y < rect.max.y {
        if y >= rect.min.y + 16.0 {
            let logical_val = ((y - origin.y) / scale).round() as i64;
            painter.line_segment(
                [Pos2::new(rect.min.x + 14.0, y), Pos2::new(rect.min.x + 20.0, y)],
                Stroke::new(1.0_f32, Color32::from_gray(140)),
            );
            painter.text(
                Pos2::new(rect.min.x + 2.0, y + 2.0),
                egui::Align2::LEFT_TOP,
                format!("{logical_val}"),
                font.clone(),
                text_col,
            );
        }
        y += step;
    }
}

/// Origin crosshair: the arms are `arm` pixels long (the original scales them with the zoom).
pub fn draw_origin(painter: &egui::Painter, origin: Pos2, arm: f32, origin_color: Color32) {
    let s = arm.max(6.0);
    let stroke = Stroke::new(1.4_f32, origin_color);
    painter.line_segment([Pos2::new(origin.x - s, origin.y), Pos2::new(origin.x + s, origin.y)], stroke);
    painter.line_segment([Pos2::new(origin.x, origin.y - s), Pos2::new(origin.x, origin.y + s)], stroke);
}

fn with_alpha(c: Color, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, alpha)
}

/// Thin editor outline of the shape (the original's `rgba_outline` stroke). Open polylines.
pub fn draw_editor_outline(
    painter: &egui::Painter,
    polylines: &[Vec<Pos2>],
    settings: &AppSettings,
) {
    let col = with_alpha(settings.editor_outline, settings.editor_outline_opacity);
    for pl in polylines {
        if pl.len() >= 2 {
            painter.add(egui::Shape::line(pl.clone(), Stroke::new(1.0_f32, col)));
        }
    }
}

/// ASS border preview: stroke of `width * scale` pixels around a (closed) contour.
pub fn draw_style_outline(
    painter: &egui::Painter,
    polyline: &[Pos2],
    width_px: f32,
    color: Color32,
) {
    if polyline.len() >= 2 && width_px > 0.0 {
        painter.add(egui::Shape::closed_line(polyline.to_vec(), Stroke::new(width_px, color)));
    }
}

/// Screen points of a command including its start (used for guides and highlighting).
fn cmd_guide_points(
    drawing: &Drawing,
    idx: usize,
    to_screen: &dyn Fn(f64, f64) -> Pos2,
) -> Vec<Pos2> {
    let mut pts = Vec::new();
    if idx > 0 {
        if let Some((x, y)) = drawing.main_pos_of_cmd(idx - 1) {
            pts.push(to_screen(x, y));
        }
    }
    for id in drawing.commands[idx].all_points() {
        if let Some(p) = drawing.points.get(&id) {
            pts.push(to_screen(p.x, p.y));
        }
    }
    pts
}

#[allow(clippy::too_many_arguments)]
pub fn draw_points(
    painter: &egui::Painter,
    drawing: &Drawing,
    mouse: &CanvasMouseState,
    origin: Pos2,
    scale: f32,
    settings: &AppSettings,
    rect: Rect,
) {
    let to_screen = |x: f64, y: f64| Pos2::new(origin.x + x as f32 * scale, origin.y + y as f32 * scale);

    // Guide lines: the control polygon of every Bezier / spline, drawn from the previous point.
    let guide = with_alpha(settings.guideline_color, settings.guideline_opacity);
    for (i, cmd) in drawing.commands.iter().enumerate() {
        if let DrawCmd::B { c1_cont, .. } = cmd {
            let col = if *c1_cont { Color32::from_rgb(0, 220, 150) } else { guide };
            let pts = cmd_guide_points(drawing, i, &to_screen);
            if pts.len() >= 2 {
                painter.add(egui::Shape::line(pts, Stroke::new(1.0_f32, col)));
            }
        } else if matches!(cmd, DrawCmd::S { .. }) {
            let pts = cmd_guide_points(drawing, i, &to_screen);
            if pts.len() >= 2 {
                painter.add(egui::Shape::line(pts, Stroke::new(1.0_f32, guide)));
            }
        }
    }

    // Highlighted command (the one under the cursor), except plain moves.
    if let Some(id) = mouse.pointed_id {
        if let Some(i) = drawing.cmd_index_of_point(id) {
            if !matches!(drawing.commands[i], DrawCmd::M { .. } | DrawCmd::N { .. }) {
                let pts = cmd_guide_points(drawing, i, &to_screen);
                if pts.len() >= 2 {
                    let col = with_alpha(settings.editor_outline, 160);
                    painter.add(egui::Shape::line(pts, Stroke::new(3.0_f32, col)));
                }
            }
        }
    }

    let diameter = scale.clamp(8.0, 14.0);
    let radius = diameter / 2.0;
    let main_col = with_alpha(settings.point_main, settings.point_opacity);
    let ctrl_col = with_alpha(settings.point_control, settings.control_point_opacity);
    let sel_col = with_alpha(settings.point_selected, settings.selected_point_opacity);

    for p in drawing.points.values() {
        let pos = to_screen(p.x, p.y);
        match p.kind {
            PointKind::Main => {
                painter.rect_filled(Rect::from_center_size(pos, egui::vec2(diameter, diameter)), 0.0, main_col);
            }
            PointKind::Control => {
                painter.circle_filled(pos, radius, ctrl_col);
            }
        }
    }
    for id in &mouse.selected {
        if let Some(p) = drawing.points.get(id) {
            painter.circle_stroke(to_screen(p.x, p.y), radius + 3.0, Stroke::new(1.5_f32, sel_col));
        }
    }

    // Hover: ring, coordinates label and cross guides through the point.
    if let Some(p) = mouse.pointed_id.and_then(|id| drawing.points.get(&id)) {
        let pos = to_screen(p.x, p.y);
        painter.circle_stroke(pos, radius + 3.0, Stroke::new(2.0_f32, sel_col));
        let cross = with_alpha(settings.guideline_color, 128);
        painter.line_segment([Pos2::new(pos.x, rect.min.y), Pos2::new(pos.x, rect.max.y)], Stroke::new(1.0_f32, cross));
        painter.line_segment([Pos2::new(rect.min.x, pos.y), Pos2::new(rect.max.x, pos.y)], Stroke::new(1.0_f32, cross));
        painter.text(
            Pos2::new(pos.x + 6.0, pos.y - 6.0),
            egui::Align2::LEFT_BOTTOM,
            format!("{},{}", p.x.round() as i64, p.y.round() as i64),
            FontId::monospace(11.0),
            with_alpha(settings.editor_outline, 255),
        );
    }

    // Selection rubber band.
    if let Some((a, b)) = mouse.marquee {
        let r = Rect::from_two_pos(a, b);
        painter.rect_stroke(r, 0.0, Stroke::new(1.0_f32, with_alpha(settings.editor_outline, 160)));
    }

    // Transform handles.
    if let Some(q) = mouse.transform_quad {
        let red = Color32::from_rgb(255, 70, 70);
        for i in 0..4 {
            let a = to_screen(q[i].0, q[i].1);
            let b = to_screen(q[(i + 1) % 4].0, q[(i + 1) % 4].1);
            painter.line_segment([a, b], Stroke::new(1.5_f32, red));
            painter.circle_stroke(a, 6.0, Stroke::new(1.5_f32, red));
        }
        if mouse.tool == Tool::ScaleRotate {
            let cx = (q[0].0 + q[2].0) * 0.5;
            let cy = (q[0].1 + q[2].1) * 0.5;
            let c = to_screen(cx, cy);
            painter.line_segment([c - egui::vec2(10.0, 10.0), c + egui::vec2(10.0, 10.0)], Stroke::new(1.0_f32, red));
            painter.line_segment([c + egui::vec2(10.0, -10.0), c + egui::vec2(-10.0, 10.0)], Stroke::new(1.0_f32, red));
            painter.circle_stroke(c, 10.0, Stroke::new(1.0_f32, red));
        }
    }

    // Spline tool: points collected so far.
    for &(x, y) in &mouse.temp_points {
        let p = to_screen(x, y);
        painter.circle_filled(p, 5.0, Color32::YELLOW);
        painter.circle_stroke(p, 5.0, Stroke::new(1.5_f32, Color32::BLACK));
    }
}
