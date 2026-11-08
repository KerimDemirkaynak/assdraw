//! Anti-aliased filled shape rendering (AGG-like quality target).
//!
//! Pipeline:
//! 1. Walk DrawCmd list → closed contours (logical coords)
//! 2. Flatten cubic Béziers to line segments
//! 3. Build a `lyon` path with the **non-zero winding** rule (AGG / libass behaviour)
//! 4. Tessellate to triangle list
//! 5. Emit `egui::Mesh` in screen space
//!
//! Holes: a contour nested in another one with the opposite winding is a hole of that sub-shape;
//! sub-shapes are tessellated one by one so every sub-shape keeps its own colour.

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

/// Build filled meshes for the drawing: one mesh per sub-shape (an outer contour together with its
/// holes), filled with the **non-zero winding** rule like AGG and libass.
///
/// `to_screen`: logical → screen. Sub-shapes without an own colour use `default_fill`; `opacity`
/// is the global fill opacity (0..=255).
pub fn build_filled_meshes(
    drawing: &Drawing,
    to_screen: &dyn Fn(f64, f64) -> Pos2,
    default_fill: Color32,
    opacity: u8,
) -> Vec<FilledShape> {
    // Flatten tolerance of 0.25 screen pixels, whatever the zoom level.
    let p0 = to_screen(0.0, 0.0);
    let p1 = to_screen(1.0, 0.0);
    let px_per_unit = f64::from((p1.x - p0.x).abs()).max(0.01);
    let contours = extract_contours(drawing, (0.25 / px_per_unit).min(FLATTEN_TOL));
    if contours.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (outer, members) in drawing.subshape_groups() {
        let fill = if let Some(st) = drawing.shape_styles.get(&outer) {
            if !st.fill_enabled {
                continue;
            }
            Color32::from_rgba_unmultiplied(st.fill.r, st.fill.g, st.fill.b, st.fill_opacity)
        } else if let Some(c) = drawing.get_contour_color(outer) {
            Color32::from_rgba_unmultiplied(c.r, c.g, c.b, opacity)
        } else {
            let a = default_fill.to_array();
            Color32::from_rgba_unmultiplied(a[0], a[1], a[2], opacity)
        };
        let batch: Vec<&Contour> = members.iter().filter_map(|&m| contours.get(m)).collect();
        if let Some(mesh) = tessellate_contours(&batch, to_screen, fill) {
            out.push(FilledShape { mesh });
        }
    }
    out
}

pub struct Contour {
    /// Flattened logical polyline (closed implicitly when filled).
    pub points: Vec<(f64, f64)>,
}

/// One flattened polyline per contour; the index matches `Drawing::subshape_groups`.
pub fn extract_contours(drawing: &Drawing, tol: f64) -> Vec<Contour> {
    let mut contours: Vec<Contour> = Vec::new();
    let mut last_point = (0.0, 0.0);
    let cmds = &drawing.commands;
    let mut idx = 0usize;
    while idx < cmds.len() {
        let cmd = &cmds[idx];
        if cmd.is_contour_start() || contours.is_empty() {
            contours.push(Contour { points: Vec::new() });
        }
        let current = &mut contours.last_mut().expect("contour exists").points;
        match cmd {
            DrawCmd::M { point } | DrawCmd::N { point } | DrawCmd::L { point } | DrawCmd::P { point } => {
                if let Some(p) = drawing.points.get(point) {
                    current.push((p.x, p.y));
                    last_point = (p.x, p.y);
                }
                idx += 1;
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
                        tol,
                        MAX_SUBDIV + 4,
                    );
                    for &pt in pts.iter().skip(1) {
                        current.push(pt);
                    }
                    last_point = (e.x, e.y);
                }
                idx += 1;
            }
            DrawCmd::S { points } => {
                let mut logical = Vec::new();
                for pid in points {
                    if let Some(p) = drawing.points.get(pid) {
                        logical.push((p.x, p.y));
                    }
                }
                idx += 1;
                while idx < cmds.len() {
                    if let DrawCmd::P { point } = &cmds[idx] {
                        if let Some(p) = drawing.points.get(point) {
                            logical.push((p.x, p.y));
                        }
                        idx += 1;
                    } else {
                        break;
                    }
                }
                if logical.len() >= 2 {
                    let flat = flatten_bspline(&logical, 0.05);
                    for &pt in &flat {
                        current.push(pt);
                    }
                    if let Some(&lp) = logical.last() {
                        last_point = lp;
                    }
                }
            }
            DrawCmd::C => {
                idx += 1;
            }
        }
    }
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

    let mut buffers: VertexBuffers<lyon::math::Point, u32> = VertexBuffers::new();
    let mut tess = FillTessellator::new();
    let opts = FillOptions::tolerance(0.2).with_fill_rule(FillRule::NonZero);
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
    for idx in &buffers.indices {
        mesh.indices.push(*idx);
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

// Shared with engine hole/winding tests (AGG conv_bspline port).
pub use crate::engine::flatten_bspline;

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
