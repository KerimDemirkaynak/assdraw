//! Canvas painting helpers: Grid, Rulers, Origin Crosshair, Paths, Points & Handles, Selection Marquee, Transform Quad.

use crate::canvas::mouse::CanvasMouseState;
use crate::engine::{Color, DrawCmd, Drawing, PointKind};
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

pub fn draw_origin(
    painter: &egui::Painter,
    origin: Pos2,
    cross_size: f32,
    origin_color: Color32,
    is_hover: bool,
) {
    let s = (cross_size * 5.0).max(10.0) * if is_hover { 1.3 } else { 1.0 };
    let stroke = Stroke::new(if is_hover { 2.2_f32 } else { 1.4_f32 }, origin_color);

    // Cross lines
    painter.line_segment([Pos2::new(origin.x - s, origin.y), Pos2::new(origin.x + s, origin.y)], stroke);
    painter.line_segment([Pos2::new(origin.x, origin.y - s), Pos2::new(origin.x, origin.y + s)], stroke);

    // Circle at origin
    painter.circle_stroke(origin, s * 0.45, stroke);
    if is_hover {
        painter.circle_filled(origin, s * 0.25, origin_color);
    }

    painter.text(
        Pos2::new(origin.x + 4.0, origin.y + 4.0),
        egui::Align2::LEFT_TOP,
        "(0,0)",
        FontId::monospace(9.0),
        origin_color,
    );
}

pub fn draw_paths(
    painter: &egui::Painter,
    drawing: &Drawing,
    to_screen: &dyn Fn(f64, f64) -> Pos2,
    default_color: Color,
    border_color: Color,
    border_width: f32,
) {
    let mut path: Vec<Pos2> = Vec::new();
    let mut contour = 0usize;
    let stroke_width = if border_width > 0.1 { border_width } else { 2.0 };

    let flush = |path: &mut Vec<Pos2>, c: usize, painter: &egui::Painter| {
        if path.len() >= 2 {
            let col = drawing
                .get_contour_color(c)
                .map(|c| c.to_egui())
                .unwrap_or(default_color.to_egui());
            painter.add(egui::Shape::line(
                std::mem::take(path),
                Stroke::new(stroke_width, if border_width > 0.1 { border_color.to_egui() } else { col }),
            ));
        } else {
            path.clear();
        }
    };

    for (idx, cmd) in drawing.commands.iter().enumerate() {
        if cmd.is_contour_start() && idx > 0 {
            flush(&mut path, contour, painter);
            contour += 1;
        }
        match cmd {
            DrawCmd::M { point } | DrawCmd::N { point } | DrawCmd::L { point } => {
                if let Some(p) = drawing.points.get(point) {
                    path.push(to_screen(p.x, p.y));
                }
            }
            DrawCmd::B { control1, control2, end, .. } => {
                if let (Some(c1), Some(c2), Some(e)) = (
                    drawing.points.get(control1),
                    drawing.points.get(control2),
                    drawing.points.get(end),
                ) {
                    let start = path.last().copied().unwrap_or(to_screen(c1.x, c1.y));
                    for i in 1..=24 {
                        let t = i as f32 / 24.0;
                        path.push(cubic(
                            start,
                            to_screen(c1.x, c1.y),
                            to_screen(c2.x, c2.y),
                            to_screen(e.x, e.y),
                            t,
                        ));
                    }
                }
            }
            DrawCmd::S { points } => {
                for pid in points {
                    if let Some(p) = drawing.points.get(pid) {
                        path.push(to_screen(p.x, p.y));
                    }
                }
            }
            DrawCmd::P { point } => {
                if let Some(p) = drawing.points.get(point) {
                    path.push(to_screen(p.x, p.y));
                }
            }
            DrawCmd::C => {
                if let Some(f) = path.first().copied() {
                    path.push(f);
                }
            }
        }
    }
    flush(&mut path, contour, painter);
}

fn cubic(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    Pos2 {
        x: uu * u * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + tt * t * p3.x,
        y: uu * u * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + tt * t * p3.y,
    }
}

pub fn draw_points(
    painter: &egui::Painter,
    drawing: &Drawing,
    mouse: &CanvasMouseState,
    to_screen: &dyn Fn(f64, f64) -> Pos2,
    settings: &AppSettings,
) {
    // Control handle lines for Bezier
    for cmd in &drawing.commands {
        if let DrawCmd::B { control1, control2, end, c1_cont } = cmd {
            if let (Some(c1), Some(c2), Some(e)) = (
                drawing.points.get(control1),
                drawing.points.get(control2),
                drawing.points.get(end),
            ) {
                let pe = to_screen(e.x, e.y);
                let line_col = if *c1_cont {
                    Color32::from_rgb(0, 220, 150)
                } else {
                    settings.guideline_color.to_egui()
                };
                painter.line_segment(
                    [to_screen(c1.x, c1.y), pe],
                    Stroke::new(1.0_f32, line_col),
                );
                painter.line_segment(
                    [to_screen(c2.x, c2.y), pe],
                    Stroke::new(1.0_f32, line_col),
                );
            }
        }
    }

    // Points (Square anchors for ASSDraw3, circles for Bezier control handles)
    for p in drawing.points.values() {
        let pos = to_screen(p.x, p.y);
        let is_hover = mouse.pointed_id == Some(p.id);
        let is_sel = mouse.selected.contains(&p.id);

        match p.kind {
            PointKind::Main => {
                let half_size = if is_hover { 5.5 } else { 4.0 };
                let p_rect = Rect::from_center_size(pos, egui::vec2(half_size * 2.0, half_size * 2.0));
                let col = Color32::from_rgba_unmultiplied(
                    settings.point_main.r,
                    settings.point_main.g,
                    settings.point_main.b,
                    settings.point_opacity,
                );
                painter.rect_filled(p_rect, 0.0, col);
                painter.rect_stroke(p_rect, 0.0, Stroke::new(1.0_f32, Color32::from_gray(30)));
            }
            PointKind::Control => {
                let r = if is_hover { 5.5 } else { 3.5 };
                let col = Color32::from_rgba_unmultiplied(
                    settings.point_control.r,
                    settings.point_control.g,
                    settings.point_control.b,
                    settings.control_point_opacity,
                );
                painter.circle_filled(pos, r, col);
                painter.circle_stroke(pos, r, Stroke::new(1.0_f32, Color32::BLACK));
            }
        }

        if is_sel {
            let sel_col = Color32::from_rgba_unmultiplied(
                settings.point_selected.r,
                settings.point_selected.g,
                settings.point_selected.b,
                settings.selected_point_opacity,
            );
            painter.circle_stroke(
                pos,
                9.5,
                Stroke::new(2.0_f32, sel_col),
            );
        }
    }

    // Marquee Selection Box
    if let Some((a, b)) = mouse.marquee {
        let rect = Rect::from_two_pos(a, b);
        painter.rect_stroke(rect, 0.0, Stroke::new(1.0_f32, Color32::from_rgb(100, 190, 255)));
        painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(80, 150, 240, 45));
    }

    // Transform Bounding Quad
    if let Some(q) = mouse.transform_quad {
        for i in 0..4 {
            let a = to_screen(q[i].0, q[i].1);
            let b = to_screen(q[(i + 1) % 4].0, q[(i + 1) % 4].1);
            painter.line_segment([a, b], Stroke::new(1.5_f32, Color32::from_rgb(255, 90, 90)));
            painter.circle_filled(a, 6.0, Color32::from_rgb(255, 70, 70));
            painter.circle_stroke(a, 6.0, Stroke::new(1.5_f32, Color32::WHITE));
        }
    }

    // Temp points for placement tools (Bezier, Spline)
    for &(x, y) in &mouse.temp_points {
        let p = to_screen(x, y);
        painter.circle_filled(p, 5.0, Color32::YELLOW);
        painter.circle_stroke(p, 5.0, Stroke::new(1.5_f32, Color32::BLACK));
    }
}
