//! Mouse state machine (mirrors ASSDrawCanvas::OnMouse*).
//! Supports independent/unified Drawing and Background drag modes,
//! origin dragging, marquee selection, transform handles, and hover hit-testing.

use crate::engine::{DrawCmd, Drawing, Id, PointKind};
use crate::ui::Tool;
use egui::{Pos2, Vec2};

/// Drag targets (mirrors DRAGMODE: DRAG_DWG, DRAG_BGIMG, DRAG_BOTH).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragTarget {
    Drawing,
    Background,
    Both,
}

impl Default for DragTarget {
    fn default() -> Self {
        DragTarget::Drawing
    }
}

/// Selection modifier (mirrors original SELECTMODE: NEW, ADD, DEL/TOGGLE).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    New,
    Add,
    Toggle,
}

/// What the left button is currently doing.
#[derive(Debug, Clone)]
pub enum LeftDragKind {
    None,
    /// Moving one or more selected points
    MovePoints { last_logical: (f64, f64) },
    /// Dragging the origin crosshair [0,0]
    MoveOrigin { last_logical: (f64, f64) },
    /// Rubber-band selection rectangle (screen space)
    Marquee { start: Pos2, current: Pos2 },
    /// Placing a new draw command (mode M/L/B/S…)
    PlaceCommand,
    /// Scale/rotate or bilinear handle
    TransformHandle { corner: usize },
}

/// Central mouse/session state for the canvas (survives across frames).
#[derive(Debug, Clone)]
pub struct CanvasMouseState {
    pub tool: Tool,
    pub drag_target: DragTarget,

    /// Point under cursor (hover)
    pub pointed_id: Option<Id>,
    /// True if hovering over origin crosshair (0,0)
    pub hover_origin: bool,
    /// Point where left button went down
    pub mouse_down_id: Option<Id>,
    pub selected: Vec<Id>,
    pub left_drag: LeftDragKind,
    pub left_down: bool,
    pub right_down: bool,

    /// Anchor for right-drag pan
    pub pan_anchor: Option<Pos2>,
    /// Drawing layer pan
    pub pan: Vec2,
    /// Drawing layer zoom factor
    pub view_zoom: f32,

    /// Background image layer pan
    pub bg_pan: Vec2,
    /// Background image layer zoom factor
    pub bg_zoom: f32,

    /// Screen-space rubber band for UI draw
    pub marquee: Option<(Pos2, Pos2)>,
    /// Status secondary text (command under cursor)
    pub status_cmd: String,
    /// Preview mode: render pure shape, hide all handles & rulers
    pub preview_mode: bool,
    /// Undo description for the in-progress gesture
    pub undo_desc: String,
    /// Temp points for multi-click tools (manual bezier, spline)
    pub temp_points: Vec<(f64, f64)>,
    /// Transform bounding quad corners in logical space (TL, TR, BR, BL)
    pub transform_quad: Option<[(f64, f64); 4]>,
}

impl Default for CanvasMouseState {
    fn default() -> Self {
        Self {
            tool: Tool::Select,
            drag_target: DragTarget::Drawing,
            pointed_id: None,
            hover_origin: false,
            mouse_down_id: None,
            selected: Vec::new(),
            left_drag: LeftDragKind::None,
            left_down: false,
            right_down: false,
            pan_anchor: None,
            pan: Vec2::ZERO,
            view_zoom: 1.0,
            bg_pan: Vec2::ZERO,
            bg_zoom: 1.0,
            marquee: None,
            status_cmd: String::new(),
            preview_mode: false,
            undo_desc: String::new(),
            temp_points: Vec::new(),
            transform_quad: None,
        }
    }
}

/// Hit-test radius in logical units given scale and DPI-ish factor.
pub fn hit_radius_logical(scale: f32, view_zoom: f32) -> f64 {
    let screen_r = 9.0_f32.max(7.0 * view_zoom);
    (screen_r / (scale * view_zoom).max(0.001)) as f64
}

impl CanvasMouseState {
    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn select_all(&mut self, drawing: &Drawing) {
        self.selected = drawing.points.keys().copied().collect();
    }

    pub fn select_mode(shift: bool, ctrl: bool) -> SelectMode {
        if ctrl {
            SelectMode::Toggle
        } else if shift {
            SelectMode::Add
        } else {
            SelectMode::New
        }
    }

    /// Find point at logical position. Controls checked first; main point wins if both hit.
    pub fn find_point_at(
        drawing: &Drawing,
        lx: f64,
        ly: f64,
        max_dist: f64,
    ) -> Option<Id> {
        let mut best_ctrl: Option<(Id, f64)> = None;
        let mut best_main: Option<(Id, f64)> = None;
        for (id, p) in &drawing.points {
            let d = p.dist_to_xy(lx, ly);
            if d > max_dist {
                continue;
            }
            match p.kind {
                PointKind::Control => {
                    if best_ctrl.map_or(true, |(_, bd)| d < bd) {
                        best_ctrl = Some((*id, d));
                    }
                }
                PointKind::Main => {
                    if best_main.map_or(true, |(_, bd)| d < bd) {
                        best_main = Some((*id, d));
                    }
                }
            }
        }
        best_main.map(|(id, _)| id).or(best_ctrl.map(|(id, _)| id))
    }

    /// Command string for status bar (original shows pointed command ToString).
    pub fn command_string_for_point(drawing: &Drawing, id: Id) -> String {
        for cmd in &drawing.commands {
            if cmd.all_points().contains(&id) {
                return cmd_to_short(drawing, cmd);
            }
        }
        String::new()
    }

    /// Apply selection mode for a single point click.
    pub fn apply_point_selection(&mut self, id: Id, mode: SelectMode) {
        match mode {
            SelectMode::New => {
                self.selected.clear();
                self.selected.push(id);
            }
            SelectMode::Add => {
                if !self.selected.contains(&id) {
                    self.selected.push(id);
                }
            }
            SelectMode::Toggle => {
                if self.selected.contains(&id) {
                    self.selected.retain(|&x| x != id);
                } else {
                    self.selected.push(id);
                }
            }
        }
    }

    /// Select all points inside a screen-space rectangle.
    pub fn select_points_in_rect(
        &mut self,
        drawing: &Drawing,
        origin: Pos2,
        scale: f32,
        a: Pos2,
        b: Pos2,
        mode: SelectMode,
    ) {
        let min_x = a.x.min(b.x);
        let max_x = a.x.max(b.x);
        let min_y = a.y.min(b.y);
        let max_y = a.y.max(b.y);
        let mut hits = Vec::new();
        for (id, p) in &drawing.points {
            let sx = origin.x + p.x as f32 * scale;
            let sy = origin.y + p.y as f32 * scale;
            if sx >= min_x && sx <= max_x && sy >= min_y && sy <= max_y {
                hits.push(*id);
            }
        }
        match mode {
            SelectMode::New => self.selected = hits,
            SelectMode::Add => {
                for id in hits {
                    if !self.selected.contains(&id) {
                        self.selected.push(id);
                    }
                }
            }
            SelectMode::Toggle => {
                for id in hits {
                    if self.selected.contains(&id) {
                        self.selected.retain(|&x| x != id);
                    } else {
                        self.selected.push(id);
                    }
                }
            }
        }
    }

    /// Initialize transform quad from drawing bounds.
    pub fn init_transform_quad(&mut self, drawing: &Drawing) {
        if let Some((min_x, min_y, max_x, max_y)) = drawing.bounds() {
            self.transform_quad = Some([
                (min_x, min_y),
                (max_x, min_y),
                (max_x, max_y),
                (min_x, max_y),
            ]);
        }
    }
}

fn cmd_to_short(drawing: &Drawing, cmd: &DrawCmd) -> String {
    match cmd {
        DrawCmd::M { point } => point_fmt(drawing, "M", *point),
        DrawCmd::N { point } => point_fmt(drawing, "N", *point),
        DrawCmd::L { point } => point_fmt(drawing, "L", *point),
        DrawCmd::B { end, .. } => point_fmt(drawing, "B", *end),
        DrawCmd::S { .. } => "S".into(),
        DrawCmd::P { point } => point_fmt(drawing, "P", *point),
        DrawCmd::C => "C".into(),
    }
}

fn point_fmt(drawing: &Drawing, name: &str, id: Id) -> String {
    if let Some(p) = drawing.points.get(&id) {
        format!("{name} {:.0},{:.0}", p.x, p.y)
    } else {
        name.to_string()
    }
}

/// Result of processing input for one frame.
#[derive(Debug, Default)]
pub struct MouseFrameResult {
    pub need_undo_push: bool,
    pub undo_label: String,
    pub status_coords: (f64, f64),
    pub status_cmd: String,
    pub request_color_dialog: bool,
    pub contour_for_color: usize,
    pub cmd_changed: bool,
    pub request_delete_sel: bool,
}
