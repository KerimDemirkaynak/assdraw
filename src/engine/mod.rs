//! Drawing engine — port of original `engine.hpp` / `engine.cpp`.
//! PointSystem, Point, DrawCmd list, parse/generate ASS, transforms.

pub mod bspline;
pub mod cmd;
pub mod convert;
pub mod history;
pub mod parse;
pub mod point;
pub mod style;
pub mod transform;

pub use bspline::flatten_bspline;
pub use cmd::*;
pub use convert::*;
pub use history::*;
pub use parse::*;
pub use point::*;
pub use style::ShapeStyle;
pub use transform::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Id = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointSystem {
    pub scale: f64,
    pub origin_x: f64,
    pub origin_y: f64,
}

impl Default for PointSystem {
    fn default() -> Self {
        Self {
            scale: 10.0,
            origin_x: 0.0,
            origin_y: 0.0,
        }
    }
}

impl PointSystem {
    pub fn to_screen(&self, x: f64, y: f64) -> (f32, f32) {
        (
            ((x - self.origin_x) * self.scale) as f32,
            ((y - self.origin_y) * self.scale) as f32,
        )
    }
    pub fn to_logical(&self, sx: f32, sy: f32) -> (f64, f64) {
        (
            sx as f64 / self.scale + self.origin_x,
            sy as f64 / self.scale + self.origin_y,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_egui(self) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(self.r, self.g, self.b, self.a)
    }

    pub fn to_egui_alpha(self, alpha: u8) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(self.r, self.g, self.b, alpha)
    }

    /// ASS BGR hex format (BBGGRR)
    pub fn to_ass_bgr(self) -> String {
        format!("{:02X}{:02X}{:02X}", self.b, self.g, self.r)
    }
}

/// Full document state (mirrors ASSDrawEngine responsibilities).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawing {
    pub points: HashMap<Id, Point>,
    pub commands: Vec<DrawCmd>,
    pub next_id: Id,
    pub point_system: PointSystem,
    /// Contour index → fill color (per-shape coloring)
    pub shape_colors: HashMap<usize, Color>,
    /// Contour index → full style (set when importing styled ASS); overrides `shape_colors`.
    #[serde(default)]
    pub shape_styles: HashMap<usize, ShapeStyle>,
    #[serde(skip)]
    pub history: History,
}

impl Default for Drawing {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawing {
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
            commands: Vec::new(),
            next_id: 1,
            point_system: PointSystem::default(),
            shape_colors: HashMap::new(),
            shape_styles: HashMap::new(),
            history: History::new(),
        }
    }

    /// Replace the shape with a copy of `other`, keeping this drawing's undo history.
    pub fn load_shape_from(&mut self, other: &Drawing) {
        self.push_history_desc("Load shape");
        let snap = other.snapshot("");
        self.apply(snap);
    }

    /// Clone points/commands/styles without history (for safe parse trials).
    pub fn clone_geometry(&self) -> Drawing {
        Drawing {
            points: self.points.clone(),
            commands: self.commands.clone(),
            next_id: self.next_id,
            shape_colors: self.shape_colors.clone(),
            shape_styles: self.shape_styles.clone(),
            point_system: self.point_system.clone(),
            history: History::new(),
        }
    }

    /// Copy geometry fields from `other` (history untouched).
    pub fn copy_geometry_from(&mut self, other: &Drawing) {
        self.points = other.points.clone();
        self.commands = other.commands.clone();
        self.next_id = other.next_id;
        self.shape_colors = other.shape_colors.clone();
        self.shape_styles = other.shape_styles.clone();
    }

    /// Remove all points/commands but keep the undo history.
    pub fn clear_shape(&mut self) {
        self.points.clear();
        self.commands.clear();
        self.shape_colors.clear();
        self.shape_styles.clear();
        self.next_id = 1;
    }

    /// Remove everything, including the undo history.
    pub fn clear(&mut self) {
        self.clear_shape();
        self.history.clear();
    }

    fn alloc_id(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_point(&mut self, x: f64, y: f64, kind: PointKind) -> Id {
        let id = self.alloc_id();
        self.points.insert(id, Point { id, x, y, kind, selected: false });
        id
    }

    pub fn cmd_m(&mut self, x: f64, y: f64) -> Id {
        let pid = self.add_point(x, y, PointKind::Main);
        self.commands.push(DrawCmd::M { point: pid });
        pid
    }

    pub fn cmd_n(&mut self, x: f64, y: f64) -> Id {
        let pid = self.add_point(x, y, PointKind::Main);
        self.commands.push(DrawCmd::N { point: pid });
        pid
    }

    pub fn cmd_l(&mut self, x: f64, y: f64) -> Id {
        let pid = self.add_point(x, y, PointKind::Main);
        self.commands.push(DrawCmd::L { point: pid });
        pid
    }

    pub fn cmd_b(&mut self, c1x: f64, c1y: f64, c2x: f64, c2y: f64, x: f64, y: f64) -> (Id, Id, Id) {
        let c1 = self.add_point(c1x, c1y, PointKind::Control);
        let c2 = self.add_point(c2x, c2y, PointKind::Control);
        let end = self.add_point(x, y, PointKind::Main);
        self.commands.push(DrawCmd::B {
            control1: c1,
            control2: c2,
            end,
            c1_cont: false,
        });
        (c1, c2, end)
    }

    /// Control points of a Bezier that starts at `p0` and ends at `p1`, laid out at 1/3 and 2/3 of
    /// the straight segment (ASSDraw3's `DrawCmd_B::Init`). Coordinates are integers.
    pub fn thirds_controls(p0: (f64, f64), p1: (f64, f64)) -> ((f64, f64), (f64, f64)) {
        let dx = ((p1.0 - p0.0) / 3.0).trunc();
        let dy = ((p1.1 - p0.1) / 3.0).trunc();
        (
            ((p0.0 + dx).round(), (p0.1 + dy).round()),
            ((p1.0 - dx).round(), (p1.1 - dy).round()),
        )
    }

    /// Index of the command that owns point `id`.
    pub fn cmd_index_of_point(&self, id: Id) -> Option<usize> {
        self.commands.iter().position(|c| c.all_points().contains(&id))
    }

    /// Main point position of the command at `idx`.
    pub fn main_pos_of_cmd(&self, idx: usize) -> Option<(f64, f64)> {
        let id = self.commands.get(idx)?.main_point()?;
        self.points.get(&id).map(|p| (p.x, p.y))
    }

    /// Create a command of `kind` (`'m'`, `'n'`, `'l'`, `'b'`) at (x, y) and insert it after command
    /// `after` (or append when `None`). Mirrors the canvas: a drawing that does not start with a
    /// move gets an `m` first. Returns (index of the new command, id of its main point).
    pub fn place_cmd(&mut self, kind: char, x: f64, y: f64, after: Option<usize>) -> (usize, Id) {
        if self.commands.is_empty() && kind != 'm' && kind != 'n' {
            self.cmd_m(x, y);
        }
        let pos = match after {
            Some(i) if i < self.commands.len() => i + 1,
            _ => self.commands.len(),
        };
        let prev = if pos > 0 { self.main_pos_of_cmd(pos - 1) } else { None };
        let (cmd, main) = match kind {
            'm' => {
                let id = self.add_point(x, y, PointKind::Main);
                (DrawCmd::M { point: id }, id)
            }
            'n' => {
                let id = self.add_point(x, y, PointKind::Main);
                (DrawCmd::N { point: id }, id)
            }
            'l' => {
                let id = self.add_point(x, y, PointKind::Main);
                (DrawCmd::L { point: id }, id)
            }
            _ => {
                let (c1, c2) = Self::thirds_controls(prev.unwrap_or((x, y)), (x, y));
                let i1 = self.add_point(c1.0, c1.1, PointKind::Control);
                let i2 = self.add_point(c2.0, c2.1, PointKind::Control);
                let id = self.add_point(x, y, PointKind::Main);
                (
                    DrawCmd::B { control1: i1, control2: i2, end: id, c1_cont: false },
                    id,
                )
            }
        };
        self.commands.insert(pos, cmd);
        (pos, main)
    }

    /// Re-generate the controls of the Bezier at `idx` at 1/3 and 2/3 (used while its end point is
    /// being dragged, like the original's not-yet-initialised B).
    pub fn reinit_bezier_controls(&mut self, idx: usize) {
        let DrawCmd::B { control1, control2, end, .. } = self.commands[idx] else { return };
        let Some(p0) = idx.checked_sub(1).and_then(|i| self.main_pos_of_cmd(i)) else { return };
        let Some(e) = self.points.get(&end).map(|p| (p.x, p.y)) else { return };
        let (c1, c2) = Self::thirds_controls(p0, e);
        if let Some(p) = self.points.get_mut(&control1) {
            p.x = c1.0;
            p.y = c1.1;
        }
        if let Some(p) = self.points.get_mut(&control2) {
            p.x = c2.0;
            p.y = c2.1;
        }
    }

    /// Append a point to the last spline (`p` command). Returns false if the drawing does not end in
    /// a spline.
    pub fn extend_last_spline(&mut self, x: f64, y: f64) -> bool {
        let mut idx = self.commands.len();
        if idx > 0 && matches!(self.commands[idx - 1], DrawCmd::C) {
            idx -= 1;
        }
        if idx == 0 || !matches!(self.commands[idx - 1], DrawCmd::S { .. }) {
            return false;
        }
        let id = self.add_point(x, y, PointKind::Main);
        if let DrawCmd::S { points } = &mut self.commands[idx - 1] {
            if let Some(&old_last) = points.last() {
                if let Some(p) = self.points.get_mut(&old_last) {
                    p.kind = PointKind::Control;
                }
            }
            points.push(id);
        }
        true
    }

    /// Close the last spline (`c`). Returns false if the drawing does not end in an open spline.
    pub fn close_last_spline(&mut self) -> bool {
        if matches!(self.commands.last(), Some(DrawCmd::S { .. })) {
            self.commands.push(DrawCmd::C);
            true
        } else {
            false
        }
    }

    /// Delete the command that owns point `id`. Like the original, the first command can only be
    /// deleted when it is the only one left. Returns false when refused.
    pub fn delete_command_of_point(&mut self, id: Id) -> bool {
        let Some(idx) = self.cmd_index_of_point(id) else { return false };
        if idx == 0 && self.commands.len() > 1 {
            return false;
        }
        self.commands.remove(idx);
        // A `c` that belonged to a removed spline would be orphaned.
        if idx > 0 && idx < self.commands.len() && matches!(self.commands[idx], DrawCmd::C)
            && !matches!(self.commands[idx - 1], DrawCmd::S { .. })
        {
            self.commands.remove(idx);
        }
        self.sweep_points();
        true
    }

    fn sweep_points(&mut self) {
        let live: std::collections::HashSet<Id> =
            self.commands.iter().flat_map(|c| c.all_points()).collect();
        self.points.retain(|pid, _| live.contains(pid));
    }

    /// Whether the Bezier ending at `point_id` has smooth-connection (C1) enabled.
    pub fn is_c1(&self, point_id: Id) -> bool {
        self.commands
            .iter()
            .any(|c| matches!(c, DrawCmd::B { end, c1_cont: true, .. } if *end == point_id))
    }

    /// ASS text of one command (`m 1 2`, `b 1 2 3 4 5 6`, ...).
    pub fn cmd_text(&self, idx: usize) -> String {
        match self.commands.get(idx) {
            Some(c) => generate_ass(std::iter::once(c), &self.points, false),
            None => String::new(),
        }
    }

    /// Round every coordinate to an integer (ASS drawings are integer based).
    pub fn round_points(&mut self) {
        for p in self.points.values_mut() {
            p.x = p.x.round();
            p.y = p.y.round();
        }
    }

    pub fn cmd_s(&mut self, coords: &[(f64, f64)]) -> Vec<Id> {
        let last = coords.len().saturating_sub(1);
        let ids: Vec<Id> = coords
            .iter()
            .enumerate()
            .map(|(i, &(x, y))| {
                self.add_point(x, y, if i == last { PointKind::Main } else { PointKind::Control })
            })
            .collect();
        self.commands.push(DrawCmd::S { points: ids.clone() });
        ids
    }

    pub fn cmd_p(&mut self, x: f64, y: f64) -> Id {
        let pid = self.add_point(x, y, PointKind::Main);
        self.commands.push(DrawCmd::P { point: pid });
        pid
    }

    pub fn cmd_c(&mut self) {
        self.commands.push(DrawCmd::C);
    }

    pub fn last_main_point(&self) -> Option<&Point> {
        for cmd in self.commands.iter().rev() {
            if let Some(id) = cmd.main_point() {
                return self.points.get(&id);
            }
        }
        None
    }

    pub fn to_ass_string(&self) -> String {
        self.to_ass_string_custom(false)
    }

    pub fn to_ass_string_custom(&self, capitalize: bool) -> String {
        generate_ass(&self.commands, &self.points, capitalize)
    }

    /// ASS line with style override tags, like the original `GenerateASS`.
    ///
    /// Neighbouring contours with the same effective style are merged into one run. Consecutive
    /// `\\p1` runs of one event are laid out side by side by the renderer, so every later run is
    /// shifted left by the width of the runs before it.
    /// `per_contour_colors == false` ignores the per-contour fill colours.
    pub fn to_styled_ass(&self, style: &ShapeStyle, capitalize: bool, per_contour_colors: bool) -> String {
        let mut contours: Vec<Vec<&DrawCmd>> = Vec::new();
        for cmd in &self.commands {
            if cmd.is_contour_start() || contours.is_empty() {
                contours.push(Vec::new());
            }
            if let Some(last) = contours.last_mut() {
                last.push(cmd);
            }
        }

        // Holes travel with their parent contour and use the parent's style.
        let subshapes = self.group_subshapes(&contours);

        let mut runs: Vec<(String, Vec<&DrawCmd>)> = Vec::new();
        for (idx, cmds) in subshapes {
            let contour_style = if !per_contour_colors {
                *style
            } else if let Some(own) = self.shape_styles.get(&idx) {
                *own
            } else {
                let mut s = *style;
                if let Some(color) = self.shape_colors.get(&idx) {
                    s.fill = *color;
                }
                s
            };
            let tags = contour_style.to_tags();
            let same = runs.last().is_some_and(|(prev, _)| *prev == tags);
            if same {
                if let Some((_, existing)) = runs.last_mut() {
                    existing.extend(cmds);
                }
            } else {
                runs.push((tags, cmds));
            }
        }

        let mut out = String::new();
        let mut advance_total = 0i64;
        for (i, (tags, cmds)) in runs.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(tags);
            out.push_str(&generate_ass_offset(
                cmds.iter().copied(),
                &self.points,
                capitalize,
                -advance_total,
            ));
            if let Some((lo, hi)) = x_range(cmds, &self.points) {
                advance_total += hi - lo;
            }
        }
        out
    }

    /// Polygon approximation (Bezier sampled 16x) of one contour, in logical coordinates.
    fn contour_polygon(&self, cmds: &[&DrawCmd]) -> Vec<(f64, f64)> {
        let pt = |id: &Id| self.points.get(id).map(|p| (p.x, p.y));
        let mut poly: Vec<(f64, f64)> = Vec::new();
        let mut i = 0usize;
        while i < cmds.len() {
            match cmds[i] {
                DrawCmd::M { point } | DrawCmd::N { point } | DrawCmd::L { point } => {
                    if let Some(p) = pt(point) {
                        poly.push(p);
                    }
                    i += 1;
                }
                DrawCmd::B { control1, control2, end, .. } => {
                    if let (Some(p1), Some(p2), Some(p3)) = (pt(control1), pt(control2), pt(end)) {
                        if let Some(&p0) = poly.last() {
                            for step in 1..=16 {
                                let t = f64::from(step) / 16.0;
                                let u = 1.0 - t;
                                poly.push((
                                    u * u * u * p0.0 + 3.0 * u * u * t * p1.0 + 3.0 * u * t * t * p2.0 + t * t * t * p3.0,
                                    u * u * u * p0.1 + 3.0 * u * u * t * p1.1 + 3.0 * u * t * t * p2.1 + t * t * t * p3.1,
                                ));
                            }
                        } else {
                            poly.push(p3);
                        }
                    }
                    i += 1;
                }
                DrawCmd::S { points: ids } => {
                    let mut logical = Vec::new();
                    for id in ids {
                        if let Some(p) = pt(id) {
                            logical.push(p);
                        }
                    }
                    i += 1;
                    while i < cmds.len() {
                        if let DrawCmd::P { point } = cmds[i] {
                            if let Some(p) = pt(point) {
                                logical.push(p);
                            }
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    if logical.len() >= 2 {
                        let flat = flatten_bspline(&logical, 0.1);
                        if let Some(&last) = poly.last() {
                            for &p in &flat {
                                if (p.0 - last.0).abs() > 1e-9 || (p.1 - last.1).abs() > 1e-9 {
                                    poly.push(p);
                                }
                            }
                        } else {
                            poly.extend(flat);
                        }
                    }
                }
                DrawCmd::P { point } => {
                    if let Some(p) = pt(point) {
                        poly.push(p);
                    }
                    i += 1;
                }
                DrawCmd::C => {
                    i += 1;
                }
            }
        }
        poly
    }

    /// Contours as command slices (a new contour starts at every `m` / `n`).
    fn contour_cmds(&self) -> Vec<Vec<&DrawCmd>> {
        let mut contours: Vec<Vec<&DrawCmd>> = Vec::new();
        for cmd in &self.commands {
            if cmd.is_contour_start() || contours.is_empty() {
                contours.push(Vec::new());
            }
            if let Some(last) = contours.last_mut() {
                last.push(cmd);
            }
        }
        contours
    }

    /// Group contours into sub-shapes like ASSDraw3's `BuildStyledSubshapes`: a hole (a contour
    /// nested in another one with opposite winding) belongs to its parent. Returns
    /// `(index of the outer contour, member contour indices)`; the outer contour comes first.
    pub fn subshape_groups(&self) -> Vec<(usize, Vec<usize>)> {
        let contours = self.contour_cmds();
        self.group_indices(&contours)
    }

    #[allow(clippy::needless_range_loop)]
    fn group_indices(&self, contours: &[Vec<&DrawCmd>]) -> Vec<(usize, Vec<usize>)> {
        let n = contours.len();
        let polys: Vec<Vec<(f64, f64)>> = contours.iter().map(|c| self.contour_polygon(c)).collect();
        let areas: Vec<f64> = polys.iter().map(|p| signed_area(p)).collect();
        let mut parent: Vec<Option<usize>> = vec![None; n];
        let mut is_hole = vec![false; n];
        for i in 0..n {
            if polys[i].len() < 3 {
                continue;
            }
            for cand in 0..n {
                if cand == i || polys[cand].len() < 3 || winding(&polys[cand], polys[i][0]) == 0 {
                    continue;
                }
                let better = match parent[i] {
                    None => true,
                    Some(p) => areas[cand].abs() < areas[p].abs(),
                };
                if better {
                    parent[i] = Some(cand);
                }
            }
            if let Some(p) = parent[i] {
                is_hole[i] = areas[i] * areas[p] < 0.0;
            }
        }
        let mut out = Vec::new();
        for i in 0..n {
            if is_hole[i] {
                continue;
            }
            let mut members = vec![i];
            for child in 0..n {
                if is_hole[child] && parent[child] == Some(i) {
                    members.push(child);
                }
            }
            out.push((i, members));
        }
        out
    }

    fn group_subshapes<'a>(&self, contours: &[Vec<&'a DrawCmd>]) -> Vec<(usize, Vec<&'a DrawCmd>)> {
        self.group_indices(contours)
            .into_iter()
            .map(|(idx, members)| {
                let mut cmds: Vec<&'a DrawCmd> = Vec::new();
                for m in members {
                    cmds.extend(contours[m].iter().copied());
                }
                (idx, cmds)
            })
            .collect()
    }

    /// Sub-shape (index of its outer contour) whose filled area (non-zero winding, holes excluded)
    /// contains the logical point. The last matching sub-shape wins, like the original.
    pub fn subshape_at(&self, lx: f64, ly: f64) -> Option<usize> {
        let contours = self.contour_cmds();
        let polys: Vec<Vec<(f64, f64)>> = contours.iter().map(|c| self.contour_polygon(c)).collect();
        let mut hit = None;
        for (outer, members) in self.group_indices(&contours) {
            let w: i32 = members.iter().map(|&m| winding(&polys[m], (lx, ly))).sum();
            if w != 0 {
                hit = Some(outer);
            }
        }
        hit
    }

    /// Bounding box of the rendered shape (flattened curves, no control points).
    pub fn shape_bounds(&self) -> Option<(f64, f64, f64, f64)> {
        let mut b: Option<(f64, f64, f64, f64)> = None;
        for c in self.contour_cmds() {
            for (x, y) in self.contour_polygon(&c) {
                b = Some(match b {
                    Some((a, bb, cc, d)) => (a.min(x), bb.min(y), cc.max(x), d.max(y)),
                    None => (x, y, x, y),
                });
            }
        }
        b
    }

    pub fn push_history(&mut self) {
        self.push_history_desc("Edit");
    }

    pub fn push_history_desc(&mut self, desc: &str) {
        self.history.push(DrawingSnapshot {
            points: self.points.clone(),
            commands: self.commands.clone(),
            next_id: self.next_id,
            shape_colors: self.shape_colors.clone(),
            shape_styles: self.shape_styles.clone(),
            desc: desc.to_string(),
        });
    }

    pub fn undo(&mut self) {
        let cur = self.snapshot("");
        if let Some(s) = self.history.undo(cur) {
            self.apply(s);
        }
    }

    pub fn redo(&mut self) {
        let cur = self.snapshot("");
        if let Some(s) = self.history.redo(cur) {
            self.apply(s);
        }
    }

    fn snapshot(&self, desc: &str) -> DrawingSnapshot {
        DrawingSnapshot {
            points: self.points.clone(),
            commands: self.commands.clone(),
            next_id: self.next_id,
            shape_colors: self.shape_colors.clone(),
            shape_styles: self.shape_styles.clone(),
            desc: desc.to_string(),
        }
    }

    fn apply(&mut self, s: DrawingSnapshot) {
        self.points = s.points;
        self.commands = s.commands;
        self.next_id = s.next_id;
        self.shape_colors = s.shape_colors;
        self.shape_styles = s.shape_styles;
    }

    /// Remove the snapshot taken by the last `push_history*` call.
    pub fn undo_discard(&mut self) {
        self.history.discard_last();
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn top_undo_desc(&self) -> Option<&str> {
        self.history.top_undo_desc()
    }

    pub fn top_redo_desc(&self) -> Option<&str> {
        self.history.top_redo_desc()
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        let mut d: Self = serde_json::from_str(s)?;
        d.history = History::new();
        Ok(d)
    }

    pub fn delete_point(&mut self, id: Id) {
        self.delete_command_of_point(id);
    }

    pub fn contour_count(&self) -> usize {
        self.commands.iter().filter(|c| c.is_contour_start()).count()
    }

    pub fn set_contour_color(&mut self, idx: usize, c: Color) {
        self.shape_colors.insert(idx, c);
        if let Some(style) = self.shape_styles.get_mut(&idx) {
            style.fill = c;
            style.fill_enabled = true;
        }
    }

    pub fn get_contour_color(&self, idx: usize) -> Option<&Color> {
        self.shape_colors.get(&idx)
    }

    pub fn hit_test_point(&self, lx: f64, ly: f64, max_dist: f64) -> Option<Id> {
        let mut best = None;
        for (id, p) in &self.points {
            let d = p.dist_to_xy(lx, ly);
            if d <= max_dist && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((*id, d));
            }
        }
        best.map(|(id, _)| id)
    }

    pub fn move_point(&mut self, id: Id, dx: f64, dy: f64) {
        if let Some(p) = self.points.get_mut(&id) {
            p.x += dx;
            p.y += dy;
        }
        self.enforce_c1_continuity(id);
    }

    pub fn set_point_pos(&mut self, id: Id, x: f64, y: f64) {
        if let Some(p) = self.points.get_mut(&id) {
            p.x = x;
            p.y = y;
        }
        self.enforce_c1_continuity(id);
    }

    pub fn move_points(&mut self, ids: &[Id], dx: f64, dy: f64) {
        for &id in ids {
            if let Some(p) = self.points.get_mut(&id) {
                p.x += dx;
                p.y += dy;
            }
        }
        // C1 must be enforced after multi-point moves too (mirrors single move_point).
        for &id in ids {
            self.enforce_c1_continuity(id);
        }
    }

    pub fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        if self.points.is_empty() {
            return None;
        }
        let (mut min_x, mut min_y) = (f64::MAX, f64::MAX);
        let (mut max_x, mut max_y) = (f64::MIN, f64::MIN);
        for p in self.points.values() {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
        Some((min_x, min_y, max_x, max_y))
    }

    pub fn center_origin(&mut self) {
        if let Some((a, b, c, d)) = self.bounds() {
            self.point_system.origin_x = (a + c) / 2.0;
            self.point_system.origin_y = (b + d) / 2.0;
        }
    }
}

fn generate_ass<'a>(
    cmds: impl IntoIterator<Item = &'a DrawCmd>,
    points: &HashMap<Id, Point>,
    capitalize: bool,
) -> String {
    generate_ass_offset(cmds, points, capitalize, 0)
}

/// Like [`generate_ass`], but adds `dx` to every (rounded) x coordinate.
fn generate_ass_offset<'a>(
    cmds: impl IntoIterator<Item = &'a DrawCmd>,
    points: &HashMap<Id, Point>,
    capitalize: bool,
    dx: i64,
) -> String {
    let mut out = String::new();
    let tag = |letter: char| -> char {
        if capitalize {
            letter.to_ascii_uppercase()
        } else {
            letter
        }
    };

    for cmd in cmds {
        if !out.is_empty() {
            out.push(' ');
        }
        match cmd {
            DrawCmd::M { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("{} {} {}", tag('m'), r(p.x) + dx, r(p.y)));
                }
            }
            DrawCmd::N { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("{} {} {}", tag('n'), r(p.x) + dx, r(p.y)));
                }
            }
            DrawCmd::L { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("{} {} {}", tag('l'), r(p.x) + dx, r(p.y)));
                }
            }
            DrawCmd::B { control1, control2, end, .. } => {
                if let (Some(c1), Some(c2), Some(e)) =
                    (points.get(control1), points.get(control2), points.get(end))
                {
                    out.push_str(&format!(
                        "{} {} {} {} {} {} {}",
                        tag('b'),
                        r(c1.x) + dx,
                        r(c1.y),
                        r(c2.x) + dx,
                        r(c2.y),
                        r(e.x) + dx,
                        r(e.y)
                    ));
                }
            }
            DrawCmd::S { points: pts } => {
                out.push(tag('s'));
                for pid in pts {
                    if let Some(p) = points.get(pid) {
                        out.push_str(&format!(" {} {}", r(p.x) + dx, r(p.y)));
                    }
                }
            }
            DrawCmd::P { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("{} {} {}", tag('p'), r(p.x) + dx, r(p.y)));
                }
            }
            DrawCmd::C => out.push(tag('c')),
        }
    }
    out
}

fn signed_area(poly: &[(f64, f64)]) -> f64 {
    let mut sum = 0.0;
    for (i, &(x1, y1)) in poly.iter().enumerate() {
        let (x2, y2) = poly[(i + 1) % poly.len()];
        sum += x1 * y2 - x2 * y1;
    }
    sum / 2.0
}

/// Non-zero winding number of `poly` around the point.
fn winding(poly: &[(f64, f64)], (px, py): (f64, f64)) -> i32 {
    if poly.len() < 3 {
        return 0;
    }
    let mut w = 0;
    for (i, &(x1, y1)) in poly.iter().enumerate() {
        let (x2, y2) = poly[(i + 1) % poly.len()];
        let side = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
        if y1 <= py {
            if y2 > py && side > 0.0 {
                w += 1;
            }
        } else if y2 <= py && side < 0.0 {
            w -= 1;
        }
    }
    w
}

/// Smallest and largest rounded x coordinate used by `cmds` (controls included).
fn x_range(cmds: &[&DrawCmd], points: &HashMap<Id, Point>) -> Option<(i64, i64)> {
    let mut range: Option<(i64, i64)> = None;
    let mut add = |id: &Id| {
        if let Some(p) = points.get(id) {
            let x = r(p.x);
            range = Some(match range {
                Some((lo, hi)) => (lo.min(x), hi.max(x)),
                None => (x, x),
            });
        }
    };
    for cmd in cmds {
        match cmd {
            DrawCmd::M { point } | DrawCmd::N { point } | DrawCmd::L { point } | DrawCmd::P { point } => {
                add(point);
            }
            DrawCmd::B { control1, control2, end, .. } => {
                add(control1);
                add(control2);
                add(end);
            }
            DrawCmd::S { points: ids } => {
                for id in ids {
                    add(id);
                }
            }
            DrawCmd::C => {}
        }
    }
    range
}

fn r(v: f64) -> i64 {
    v.round() as i64
}
