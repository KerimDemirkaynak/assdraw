//! Matrix transformation, scaling, rotation, bilinear deformation,
//! and shape repositioning (mirrors original ASSDrawEngine::Transform and dlgctrl.cpp templates).

use super::{Drawing, Id, Point};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReposAnchor {
    TopLeft,
    TopRight,
    Center,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub struct MatrixTransform {
    pub m11: f64,
    pub m12: f64,
    pub m21: f64,
    pub m22: f64,
    pub mx: f64,
    pub my: f64,
    pub nx: f64,
    pub ny: f64,
}

impl Default for MatrixTransform {
    fn default() -> Self {
        Self {
            m11: 1.0,
            m12: 0.0,
            m21: 0.0,
            m22: 1.0,
            mx: 0.0,
            my: 0.0,
            nx: 0.0,
            ny: 0.0,
        }
    }
}

impl MatrixTransform {
    pub fn template_rot_90_cw() -> Self {
        Self { m11: 0.0, m12: -1.0, m21: 1.0, m22: 0.0, mx: 1.0, my: 2.0, nx: 1.0, ny: 2.0 }
    }
    pub fn template_rot_90_ccw() -> Self {
        Self { m11: 0.0, m12: 1.0, m21: -1.0, m22: 0.0, mx: -1.0, my: 2.0, nx: -1.0, ny: 2.0 }
    }
    pub fn template_rot_180() -> Self {
        Self { m11: -1.0, m12: 0.0, m21: 0.0, m22: -1.0, mx: 0.0, my: 0.0, nx: 0.0, ny: 0.0 }
    }
    pub fn template_flip_h() -> Self {
        Self { m11: -1.0, m12: 0.0, m21: 0.0, m22: 1.0, mx: 4.0, my: 0.0, nx: 4.0, ny: 0.0 }
    }
    pub fn template_flip_v() -> Self {
        Self { m11: 1.0, m12: 0.0, m21: 0.0, m22: -1.0, mx: 0.0, my: 3.0, nx: 0.0, ny: 3.0 }
    }
    pub fn template_move_down() -> Self {
        Self { m11: 1.0, m12: 0.0, m21: 0.0, m22: 1.0, mx: 0.0, my: 0.0, nx: 0.0, ny: 5.0 }
    }
    pub fn template_move_right() -> Self {
        Self { m11: 1.0, m12: 0.0, m21: 0.0, m22: 1.0, mx: 0.0, my: 0.0, nx: 5.0, ny: 0.0 }
    }
    pub fn template_scale_h2() -> Self {
        Self { m11: 2.0, m12: 0.0, m21: 0.0, m22: 1.0, mx: 0.0, my: 0.0, nx: 0.0, ny: 0.0 }
    }
    pub fn template_scale_v3() -> Self {
        Self { m11: 1.0, m12: 0.0, m21: 0.0, m22: 3.0, mx: 0.0, my: 0.0, nx: 0.0, ny: 0.0 }
    }

    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        let dx = x - self.mx;
        let dy = y - self.my;
        let nx = self.m11 * dx + self.m12 * dy + self.nx;
        let ny = self.m21 * dx + self.m22 * dy + self.ny;
        (nx, ny)
    }
}

impl Drawing {
    /// Apply arbitrary matrix transformation to all points (or selected points if any selected).
    pub fn apply_matrix_transform(&mut self, t: &MatrixTransform, selected_only: bool, selected: &[u64]) {
        for p in self.points.values_mut() {
            if !selected_only || selected.contains(&p.id) {
                let (nx, ny) = t.transform_point(p.x, p.y);
                p.x = nx.round();
                p.y = ny.round();
            }
        }
    }

    /// Scale and rotate points around the bounding box center.
    pub fn apply_scale_rotate(&mut self, scale: f64, angle_deg: f64) {
        let Some((min_x, min_y, max_x, max_y)) = self.shape_bounds() else { return; };
        let cx = (min_x + max_x) / 2.0;
        let cy = (min_y + max_y) / 2.0;
        let rad = angle_deg.to_radians();
        let (cos, sin) = (rad.cos(), rad.sin());
        for p in self.points.values_mut() {
            let dx = p.x - cx;
            let dy = p.y - cy;
            let rx = dx * cos - dy * sin;
            let ry = dx * sin + dy * cos;
            p.x = (cx + rx * scale).round();
            p.y = (cy + ry * scale).round();
        }
    }

    /// Reposition the drawing so that the given anchor aligns with (0,0).
    pub fn reposition_shape(&mut self, anchor: ReposAnchor) {
        let Some((min_x, min_y, max_x, max_y)) = self.bounds() else { return; };
        let (shift_x, shift_y) = match anchor {
            ReposAnchor::TopLeft => (-min_x, -min_y),
            ReposAnchor::TopRight => (-max_x, -min_y),
            ReposAnchor::Center => (-(min_x + max_x) / 2.0, -(min_y + max_y) / 2.0),
            ReposAnchor::BottomLeft => (-min_x, -max_y),
            ReposAnchor::BottomRight => (-max_x, -max_y),
        };
        for p in self.points.values_mut() {
            p.x += shift_x;
            p.y += shift_y;
        }
    }

    /// Move shape so min corner is at (0,0).
    pub fn move_to_origin(&mut self) {
        self.reposition_shape(ReposAnchor::TopLeft);
    }

    /// Simple bilinear: map unit square corners. `corners` = [tl, tr, br, bl] in logical coords.
    pub fn apply_bilinear(&mut self, corners: [(f64, f64); 4]) {
        let Some((min_x, min_y, max_x, max_y)) = self.bounds() else { return; };
        let w = (max_x - min_x).max(1e-9);
        let h = (max_y - min_y).max(1e-9);
        let [tl, tr, br, bl] = corners;
        for p in self.points.values_mut() {
            let u = (p.x - min_x) / w;
            let v = (p.y - min_y) / h;
            let top_x = tl.0 + (tr.0 - tl.0) * u;
            let top_y = tl.1 + (tr.1 - tl.1) * u;
            let bot_x = bl.0 + (br.0 - bl.0) * u;
            let bot_y = bl.1 + (br.1 - bl.1) * u;
            p.x = top_x + (bot_x - top_x) * v;
            p.y = top_y + (bot_y - top_y) * v;
        }
    }
    /// Bilinear map of `original` points from source `rect` corners `[tl, tr, br, bl]` to
    /// destination `quad`. Works for axis-aligned **and** rotated/skewed source quads
    /// (inverse bilinear → forward bilinear). Always maps from the untouched originals so
    /// repeated calls while dragging do not accumulate distortion.
    pub fn apply_bilinear_from(
        &mut self,
        original: &HashMap<Id, Point>,
        rect: [(f64, f64); 4],
        quad: [(f64, f64); 4],
    ) {
        for (id, src) in original {
            if let Some(p) = self.points.get_mut(id) {
                let (u, v) = inverse_bilinear(rect, (src.x, src.y));
                let (nx, ny) = forward_bilinear(quad, u, v);
                p.x = nx.round();
                p.y = ny.round();
            }
        }
    }
}

/// Bilinear interpolation on a quad `[tl, tr, br, bl]`.
fn forward_bilinear(q: [(f64, f64); 4], u: f64, v: f64) -> (f64, f64) {
    let [tl, tr, br, bl] = q;
    let top_x = tl.0 + (tr.0 - tl.0) * u;
    let top_y = tl.1 + (tr.1 - tl.1) * u;
    let bot_x = bl.0 + (br.0 - bl.0) * u;
    let bot_y = bl.1 + (br.1 - bl.1) * u;
    (
        top_x + (bot_x - top_x) * v,
        top_y + (bot_y - top_y) * v,
    )
}

/// Inverse bilinear: find `(u, v)` in `[0,1]²` such that `forward_bilinear(q, u, v) ≈ p`.
/// Uses a few Newton iterations; falls back to axis-aligned UV if the solve fails.
fn inverse_bilinear(q: [(f64, f64); 4], p: (f64, f64)) -> (f64, f64) {
    let [tl, tr, br, bl] = q;
    // Fast path: axis-aligned rectangle (common case at transform start).
    let axis_aligned = (tl.1 - tr.1).abs() < 1e-9
        && (bl.1 - br.1).abs() < 1e-9
        && (tl.0 - bl.0).abs() < 1e-9
        && (tr.0 - br.0).abs() < 1e-9;
    if axis_aligned {
        let u = if (tr.0 - tl.0).abs() > 1e-12 {
            (p.0 - tl.0) / (tr.0 - tl.0)
        } else {
            0.0
        };
        let v = if (bl.1 - tl.1).abs() > 1e-12 {
            (p.1 - tl.1) / (bl.1 - tl.1)
        } else {
            0.0
        };
        return (u, v);
    }

    // Newton on F(u,v) = bilinear(q,u,v) - p
    let mut u = 0.5;
    let mut v = 0.5;
    for _ in 0..8 {
        let (x, y) = forward_bilinear(q, u, v);
        let fx = x - p.0;
        let fy = y - p.1;
        if fx * fx + fy * fy < 1e-14 {
            break;
        }
        // Jacobian via finite differences
        let eps = 1e-6;
        let (x_u, y_u) = forward_bilinear(q, u + eps, v);
        let (x_v, y_v) = forward_bilinear(q, u, v + eps);
        let j11 = (x_u - x) / eps;
        let j21 = (y_u - y) / eps;
        let j12 = (x_v - x) / eps;
        let j22 = (y_v - y) / eps;
        let det = j11 * j22 - j12 * j21;
        if det.abs() < 1e-14 {
            break;
        }
        u -= (j22 * fx - j12 * fy) / det;
        v -= (-j21 * fx + j11 * fy) / det;
    }
    (u, v)
}
