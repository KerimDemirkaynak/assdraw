//! Drawing engine — port of original `engine.hpp` / `engine.cpp`.
//! PointSystem, Point, DrawCmd list, parse/generate ASS, transforms.

pub mod cmd;
pub mod convert;
pub mod history;
pub mod parse;
pub mod point;
pub mod transform;

pub use cmd::*;
pub use convert::*;
pub use history::*;
pub use parse::*;
pub use point::*;
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
            history: History::new(),
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.commands.clear();
        self.shape_colors.clear();
        self.next_id = 1;
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

    pub fn cmd_b_auto(&mut self, x: f64, y: f64, smoothness: f64) -> Option<(Id, Id, Id)> {
        let prev = self.last_main_point()?;
        let (px, py) = (prev.x, prev.y);
        let dx = x - px;
        let dy = y - py;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let nx = -dy / len * smoothness * 25.0;
        let ny = dx / len * smoothness * 25.0;
        Some(self.cmd_b(
            px + dx * 0.33 + nx,
            py + dy * 0.33 + ny,
            px + dx * 0.66 - nx,
            py + dy * 0.66 - ny,
            x,
            y,
        ))
    }

    pub fn cmd_s(&mut self, coords: &[(f64, f64)]) -> Vec<Id> {
        let ids: Vec<Id> = coords
            .iter()
            .map(|&(x, y)| self.add_point(x, y, PointKind::Main))
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

    pub fn to_full_ass_line(&self, fill: &Color, border: &Color, border_width: f64, capitalize: bool) -> String {
        format!(
            "{{\\an7\\pos(0,0)\\p1\\bord{}\\1c&H{}&\\3c&H{}&}}{}{{\\p0}}",
            border_width,
            fill.to_ass_bgr(),
            border.to_ass_bgr(),
            self.to_ass_string_custom(capitalize)
        )
    }

    pub fn to_multicolor_ass(&self, default_fill: &Color, border: &Color, border_width: f64, capitalize: bool) -> String {
        if self.shape_colors.is_empty() || self.contour_count() <= 1 {
            return self.to_full_ass_line(default_fill, border, border_width, capitalize);
        }
        let mut out = String::new();
        let mut batch: Vec<&DrawCmd> = Vec::new();
        let mut cidx = 0usize;
        let flush = |batch: &[&DrawCmd], cidx: usize, out: &mut String, this: &Drawing| {
            if batch.is_empty() {
                return;
            }
            let color = this.shape_colors.get(&cidx).copied().unwrap_or(*default_fill);
            let draw = generate_ass(batch.iter().copied(), &this.points, capitalize);
            out.push_str(&format!(
                "{{\\an7\\pos(0,0)\\p1\\bord{}\\1c&H{}&\\3c&H{}&}}{}{{\\p0}}",
                border_width,
                color.to_ass_bgr(),
                border.to_ass_bgr(),
                draw
            ));
        };
        for cmd in &self.commands {
            if cmd.is_contour_start() && !batch.is_empty() {
                flush(&batch, cidx, &mut out, self);
                batch.clear();
                cidx += 1;
            }
            batch.push(cmd);
        }
        flush(&batch, cidx, &mut out, self);
        out
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
            desc: desc.to_string(),
        }
    }

    fn apply(&mut self, s: DrawingSnapshot) {
        self.points = s.points;
        self.commands = s.commands;
        self.next_id = s.next_id;
        self.shape_colors = s.shape_colors;
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
        self.points.remove(&id);
        self.commands.retain(|c| !c.all_points().contains(&id));
    }

    pub fn contour_count(&self) -> usize {
        self.commands.iter().filter(|c| c.is_contour_start()).count()
    }

    pub fn set_contour_color(&mut self, idx: usize, c: Color) {
        self.shape_colors.insert(idx, c);
    }

    pub fn get_contour_color(&self, idx: usize) -> Option<&Color> {
        self.shape_colors.get(&idx)
    }

    pub fn hit_test_point(&self, lx: f64, ly: f64, max_dist: f64) -> Option<Id> {
        let mut best = None;
        for (id, p) in &self.points {
            let d = p.dist_to_xy(lx, ly);
            if d <= max_dist && best.map_or(true, |(_, bd)| d < bd) {
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
                    out.push_str(&format!("{} {} {}", tag('m'), r(p.x), r(p.y)));
                }
            }
            DrawCmd::N { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("{} {} {}", tag('n'), r(p.x), r(p.y)));
                }
            }
            DrawCmd::L { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("{} {} {}", tag('l'), r(p.x), r(p.y)));
                }
            }
            DrawCmd::B { control1, control2, end, .. } => {
                if let (Some(c1), Some(c2), Some(e)) =
                    (points.get(control1), points.get(control2), points.get(end))
                {
                    out.push_str(&format!(
                        "{} {} {} {} {} {} {}",
                        tag('b'),
                        r(c1.x),
                        r(c1.y),
                        r(c2.x),
                        r(c2.y),
                        r(e.x),
                        r(e.y)
                    ));
                }
            }
            DrawCmd::S { points: pts } => {
                out.push(tag('s'));
                for pid in pts {
                    if let Some(p) = points.get(pid) {
                        out.push_str(&format!(" {} {}", r(p.x), r(p.y)));
                    }
                }
            }
            DrawCmd::P { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("{} {} {}", tag('p'), r(p.x), r(p.y)));
                }
            }
            DrawCmd::C => out.push(tag('c')),
        }
    }
    out
}

fn r(v: f64) -> i64 {
    v.round() as i64
}
