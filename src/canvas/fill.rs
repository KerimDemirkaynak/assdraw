//! Anti-aliased filled shape rendering (AGG-like quality target).
//!
//! Pipeline:
//! 1. Walk DrawCmd list → closed contours (logical coords)
//! 2. Flatten cubic Béziers to line segments
//! 3. Build a `lyon` path with **even-odd** fill rule (ASS / SVG style for holes)
//! 4. Tessellate to triangle list
//! 5. Emit `egui::Mesh` in screen space
//!
//! Holes: subsequent contours that lie inside a previous one are handled by
//! even-odd automatically when all contours are added to one path.

use crate::engine::{DrawCmd, Drawing};
use egui::{Color32, Mesh, Pos2, TextureId};
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    FillOptions, FillRule, FillTessellator, VertexBuffers,
};

/// One filled contour group with a solid color.
pub struct FilledShape {
    pub mesh: Mesh,
}

/// Flatten tolerance in **logical** units (smaller = smoother curves).
const FLATTEN_TOL: f64 = 0.35;
/// Max subdivisions for a cubic.
const MAX_SUBDIV: u32 = 8;

/// Build filled meshes for the drawing.
///
/// `to_screen`: logical → screen.
/// Contours without explicit colors use `default_fill`.
pub fn build_filled_meshes(
    drawing: &Drawing,
    to_screen: &dyn Fn(f64, f64) -> Pos2,
    default_fill: Color32,
    opacity: u8,
) -> Vec<FilledShape> {
    let contours = extract_contours(drawing);
    if contours.is_empty() {
        return Vec::new();
    }

    // Group consecutive contours that share the same color into one even-odd path
    // so holes (inner contours) subtract correctly.
    let mut out = Vec::new();
    let mut i = 0;
    while i < contours.len() {
        let color_idx = contours[i].color_index;
        let mut batch = vec![&contours[i]];
        i += 1;
        while i < contours.len() && contours[i].color_index == color_idx {
            batch.push(&contours[i]);
            i += 1;
        }
        let fill = drawing
            .get_contour_color(color_idx)
            .map(|c| {
                Color32::from_rgba_unmultiplied(c.r, c.g, c.b, opacity.min(c.a))
            })
            .unwrap_or_else(|| {
                let a = default_fill.to_array();
                Color32::from_rgba_unmultiplied(a[0], a[1], a[2], opacity)
            });

        if let Some(mesh) = tessellate_contours(&batch, to_screen, fill) {
            out.push(FilledShape { mesh });
        }
    }
    out
}

struct Contour {
    /// Logical polyline (already flattened), assumed closed if first≈last or closed flag
    points: Vec<(f64, f64)>,
    closed: bool,
    color_index: usize,
}

fn extract_contours(drawing: &Drawing) -> Vec<Contour> {
    let mut contours = Vec::new();
    let mut current: Vec<(f64, f64)> = Vec::new();
    let mut closed = false;
    let mut color_index = 0usize;
    let mut contour_count = 0usize;

    let flush = |current: &mut Vec<(f64, f64)>, closed: bool, color_index: usize, contours: &mut Vec<Contour>| {
        if current.len() >= 3 {
            // Ensure closed ring for fill
            if let (Some(&first), Some(&last)) = (current.first(), current.last()) {
                if (first.0 - last.0).abs() > 1e-6 || (first.1 - last.1).abs() > 1e-6 {
                    current.push(first);
                }
            }
            contours.push(Contour {
                points: std::mem::take(current),
                closed: true,
                color_index,
            });
        } else {
            current.clear();
        }
        let _ = closed;
    };

    let mut last_point = (0.0, 0.0);

    for (idx, cmd) in drawing.commands.iter().enumerate() {
        if cmd.is_contour_start() && idx > 0 {
            flush(&mut current, closed, color_index, &mut contours);
            contour_count += 1;
            color_index = contour_count;
            closed = false;
        }
        match cmd {
            DrawCmd::M { point } | DrawCmd::N { point } => {
                if let Some(p) = drawing.points.get(point) {
                    current.push((p.x, p.y));
                    last_point = (p.x, p.y);
                }
            }
            DrawCmd::L { point } => {
                if let Some(p) = drawing.points.get(point) {
                    current.push((p.x, p.y));
                    last_point = (p.x, p.y);
                }
            }
            DrawCmd::B { control1, control2, end, .. } => {
                if let (Some(c1), Some(c2), Some(e)) = (
                    drawing.points.get(control1),
                    drawing.points.get(control2),
                    drawing.points.get(end),
                ) {
                    let pts = flatten_cubic(
                        last_point,
                        (c1.x, c1.y),
                        (c2.x, c2.y),
                        (e.x, e.y),
                        FLATTEN_TOL,
                        MAX_SUBDIV,
                    );
                    // skip first (duplicate of last_point)
                    for &pt in pts.iter().skip(1) {
                        current.push(pt);
                    }
                    last_point = (e.x, e.y);
                }
            }
            DrawCmd::S { points } => {
                let mut logical = Vec::new();
                for pid in points {
                    if let Some(p) = drawing.points.get(pid) {
                        logical.push((p.x, p.y));
                    }
                }
                if logical.len() >= 2 {
                    let flat = flatten_catmull(&logical, 12);
                    for &pt in flat.iter().skip(1) {
                        current.push(pt);
                    }
                    if let Some(&lp) = logical.last() {
                        last_point = lp;
                    }
                }
            }
            DrawCmd::P { point } => {
                if let Some(p) = drawing.points.get(point) {
                    current.push((p.x, p.y));
                    last_point = (p.x, p.y);
                }
            }
            DrawCmd::C => {
                closed = true;
                if let Some(&first) = current.first() {
                    current.push(first);
                    last_point = first;
                }
            }
        }
    }
    flush(&mut current, closed, color_index, &mut contours);
    contours
}

fn tessellate_contours(
    contours: &[&Contour],
    to_screen: &dyn Fn(f64, f64) -> Pos2,
    color: Color32,
) -> Option<Mesh> {
    let mut builder = Path::builder();
    let mut any = false;
    for c in contours {
        if c.points.len() < 3 {
            continue;
        }
        let mut iter = c.points.iter();
        if let Some(&(x, y)) = iter.next() {
            let sp = to_screen(x, y);
            builder.begin(point(sp.x, sp.y));
            any = true;
            for &(x, y) in iter {
                let sp = to_screen(x, y);
                builder.line_to(point(sp.x, sp.y));
            }
            builder.end(true); // closed
        }
    }
    if !any {
        return None;
    }
    let path = builder.build();

    let mut buffers: VertexBuffers<lyon::math::Point, u16> = VertexBuffers::new();
    let mut tess = FillTessellator::new();
    let opts = FillOptions::tolerance(0.2).with_fill_rule(FillRule::EvenOdd);
    if tess
        .tessellate_path(
            &path,
            &opts,
            &mut lyon::tessellation::BuffersBuilder::new(
                &mut buffers,
                |v: lyon::tessellation::FillVertex| v.position(),
            ),
        )
        .is_err()
    {
        return None;
    }
    if buffers.vertices.is_empty() || buffers.indices.is_empty() {
        return None;
    }

    let mut mesh = Mesh::default();
    mesh.texture_id = TextureId::default(); // white texture → solid color
    for v in &buffers.vertices {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: Pos2::new(v.x, v.y),
            uv: Pos2::new(0.0, 0.0),
            color,
        });
    }
    // egui Mesh indices are u32
    for idx in &buffers.indices {
        mesh.indices.push(*idx as u32);
    }
    Some(mesh)
}

/// Recursive cubic subdivision until flatness < tol.
fn flatten_cubic(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    tol: f64,
    max_depth: u32,
) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    out.push(p0);
    flatten_cubic_rec(p0, p1, p2, p3, tol, max_depth, &mut out);
    out
}

fn flatten_cubic_rec(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    tol: f64,
    depth: u32,
    out: &mut Vec<(f64, f64)>,
) {
    // Flatness: distance of controls from chord
    let d1 = point_line_dist(p1, p0, p3);
    let d2 = point_line_dist(p2, p0, p3);
    if depth == 0 || d1.max(d2) <= tol {
        out.push(p3);
        return;
    }
    // De Casteljau split
    let m01 = mid(p0, p1);
    let m12 = mid(p1, p2);
    let m23 = mid(p2, p3);
    let m012 = mid(m01, m12);
    let m123 = mid(m12, m23);
    let m0123 = mid(m012, m123);
    flatten_cubic_rec(p0, m01, m012, m0123, tol, depth - 1, out);
    flatten_cubic_rec(m0123, m123, m23, p3, tol, depth - 1, out);
}

fn mid(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5)
}

fn point_line_dist(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-12 {
        return ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
    }
    ((p.0 - a.0) * dy - (p.1 - a.1) * dx).abs() / len
}

fn flatten_catmull(points: &[(f64, f64)], samples: usize) -> Vec<(f64, f64)> {
    if points.len() < 2 {
        return points.to_vec();
    }
    // Reuse simple sampling
    let mut out = Vec::new();
    out.push(points[0]);
    for i in 0..points.len() - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < points.len() {
            points[i + 2]
        } else {
            points[i + 1]
        };
        for s in 1..=samples {
            let t = s as f64 / samples as f64;
            out.push(catmull(p0, p1, p2, p3, t));
        }
    }
    out
}

fn catmull(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let t2 = t * t;
    let t3 = t2 * t;
    let x = 0.5
        * ((2.0 * p1.0)
            + (-p0.0 + p2.0) * t
            + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2
            + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);
    let y = 0.5
        * ((2.0 * p1.1)
            + (-p0.1 + p2.1) * t
            + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2
            + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::parse::parse_ass_into;
    use crate::engine::Drawing;

    #[test]
    fn filled_rect_produces_triangles() {
        let mut d = Drawing::new();
        parse_ass_into(&mut d, "m 0 0 l 100 0 l 100 50 l 0 50 l 0 0").unwrap();
        let meshes = build_filled_meshes(&d, &|x, y| Pos2::new(x as f32, y as f32), Color32::WHITE, 255);
        assert!(!meshes.is_empty());
        assert!(meshes[0].mesh.indices.len() >= 3);
    }

    #[test]
    fn cubic_flattens_to_multiple_points() {
        let pts = flatten_cubic((0.0, 0.0), (0.0, 50.0), (50.0, 50.0), (50.0, 0.0), 0.5, 8);
        assert!(pts.len() > 4);
    }
}
