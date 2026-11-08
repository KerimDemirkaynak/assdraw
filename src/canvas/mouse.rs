//! Mouse state shared by the canvas (mirrors the fields of ASSDrawCanvas that drive
//! `OnMouse*`): selection, hover, drag gestures, view pan/zoom, transform handles.

use crate::engine::{Drawing, Id, Point, PointKind};
use crate::ui::Tool;
use egui::{Pos2, Vec2};
use std::collections::HashMap;

/// Drag targets (mirrors DRAGMODE: DRAG_DWG, DRAG_BGIMG, both).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DragTarget {
    #[default]
    Drawing,
    Background,
    Both,
}

impl DragTarget {
    pub fn drawing(self) -> bool {
        matches!(self, DragTarget::Drawing | DragTarget::Both)
    }
    pub fn background(self) -> bool {
        matches!(self, DragTarget::Background | DragTarget::Both)
    }
}

/// Selection modifier (original SELECTMODE): Ctrl adds, Alt removes, otherwise a new selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    New,
    Add,
    Remove,
}

/// What the left button is currently doing.
#[derive(Debug, Clone)]
pub enum LeftDragKind {
    None,
    /// Dragging selected points (or the clicked point).
    MovePoints,
    /// Dragging the origin crosshair [0,0].
    MoveOrigin,
    /// Rubber-band selection rectangle (screen space).
    Marquee { start: Pos2, current: Pos2 },
    /// A command was just created by this press and its end point follows the mouse.
    PlaceCommand { cmd_idx: usize, point: Id },
    /// Scale/rotate or bilinear handle. `edge_partner` is Some(j) when an edge (corner..j) is dragged.
    TransformHandle {
        corner: usize,
        edge_partner: Option<usize>,
        /// Mouse position (logical) when the drag started.
        start_logical: (f64, f64),
        /// Mouse position (logical) at the previous frame (incremental edge moves in bilinear mode).
        last_logical: (f64, f64),
    },
}

/// Points of the shape and its bounding rectangle at the moment the transform tool was entered.
pub type TransformEntry = (HashMap<Id, Point>, [(f64, f64); 4]);

/// Central mouse/session state for the canvas (survives across frames).
#[derive(Debug, Clone)]
pub struct CanvasMouseState {
    /// Logical position of the last right-click (where the context menu was opened).
    pub ctx_logical: (f64, f64),
    /// Point that was under the cursor when the context menu was opened.
    pub ctx_point: Option<Id>,
    pub tool: Tool,
    pub drag_target: DragTarget,

    /// Point under cursor (hover)
    pub pointed_id: Option<Id>,
    /// Point where the left button went down
    pub mouse_down_id: Option<Id>,
    pub selected: Vec<Id>,
    pub left_drag: LeftDragKind,
    pub left_down: bool,

    /// Drawing layer pan (the origin [0,0] sits at canvas centre + pan)
    pub pan: Vec2,
    /// Drawing zoom = screen pixels per drawing unit (1..=50, original range)
    pub view_zoom: f32,

    /// Background image layer pan / zoom
    pub bg_pan: Vec2,
    pub bg_zoom: f32,

    /// Screen-space rubber band for UI draw
    pub marquee: Option<(Pos2, Pos2)>,
    /// Status secondary text (command under cursor)
    pub status_cmd: String,
    /// Preview mode: render the pure shape, hide all handles & rulers
    pub preview_mode: bool,
    /// Undo description for the in-progress gesture (empty = nothing recorded yet)
    pub undo_desc: String,
    /// Temp points for the multi-click spline tool
    pub temp_points: Vec<(f64, f64)>,
    /// Transform bounding quad corners in logical space (TL, TR, BR, BL)
    pub transform_quad: Option<[(f64, f64); 4]>,
    /// Shape + rectangle when the transform tool was entered (every drag maps from these).
    pub transform_entry: Option<TransformEntry>,
    /// Quad at the start of the current drag (the original's `backup`).
    pub transform_start_quad: [(f64, f64); 4],
    /// Right-drag rotate: logical mouse position at press.
    pub rotate_start_logical: Option<(f64, f64)>,
    /// Right-drag rotate: centre and radius at press.
    pub rotate_center_radius: Option<((f64, f64), f64)>,
    /// Coloring tool: sub-shape (index of its outer contour) selected by the last click.
    pub coloring_target: Option<usize>,
}

impl Default for CanvasMouseState {
    fn default() -> Self {
        Self {
            ctx_logical: (0.0, 0.0),
            ctx_point: None,
            tool: Tool::Select,
            drag_target: DragTarget::Drawing,
            pointed_id: None,
            mouse_down_id: None,
            selected: Vec::new(),
            left_drag: LeftDragKind::None,
            left_down: false,
            pan: Vec2::ZERO,
            view_zoom: DEFAULT_SCALE,
            bg_pan: Vec2::ZERO,
            bg_zoom: 1.0,
            marquee: None,
            status_cmd: String::new(),
            preview_mode: false,
            undo_desc: String::new(),
            temp_points: Vec::new(),
            transform_quad: None,
            transform_entry: None,
            transform_start_quad: [(0.0, 0.0); 4],
            rotate_start_logical: None,
            rotate_center_radius: None,
            coloring_target: None,
        }
    }
}

/// Original DEFAULT_SCALE.
pub const DEFAULT_SCALE: f32 = 10.0;
pub const MIN_SCALE: f32 = 1.0;
pub const MAX_SCALE: f32 = 50.0;
/// Hit radius of points in screen pixels (kEditorPointHitRadius).
pub const POINT_HIT_RADIUS: f32 = 9.0;
/// Hit radius of transform handles in screen pixels (kTransformHandleRadius).
pub const HANDLE_HIT_RADIUS: f32 = 8.0;

impl CanvasMouseState {
    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn select_all(&mut self, drawing: &Drawing) {
        self.selected = drawing.points.keys().copied().collect();
    }

    /// Screen position of the drawing origin [0,0] inside `rect`.
    pub fn origin(&self, rect: egui::Rect) -> Pos2 {
        rect.center() + self.pan
    }

    /// Pan by `d` screen pixels, honouring the "pan drawing / pan background" toggles.
    pub fn pan_by(&mut self, d: Vec2) {
        if self.drag_target.drawing() {
            self.pan += d;
            if !self.drag_target.background() {
                // The background stays where it is on screen.
                self.bg_pan -= d;
            }
        } else if self.drag_target.background() {
            self.bg_pan += d;
        }
    }

    /// Change the zoom by `amount` (original semantics: added to the scale, 1..=50; the background
    /// zooms by amount/10). The point under `anchor` stays fixed on screen.
    pub fn zoom_by(&mut self, rect: egui::Rect, amount: f32, anchor: Option<Pos2>) {
        if self.drag_target.drawing() {
            let old = self.view_zoom;
            let new = (old + amount).clamp(MIN_SCALE, MAX_SCALE);
            let origin = self.origin(rect);
            let before = self.pan;
            if let Some(pos) = anchor {
                let lx = (pos.x - origin.x) / old;
                let ly = (pos.y - origin.y) / old;
                self.pan.x += lx * (old - new);
                self.pan.y += ly * (old - new);
            }
            self.view_zoom = new;
            if self.drag_target.background() {
                let ratio = new / old;
                self.bg_zoom = (self.bg_zoom * ratio).max(0.01);
                self.bg_pan *= ratio;
            } else {
                self.bg_pan -= self.pan - before;
            }
        } else if self.drag_target.background() {
            let old = self.bg_zoom;
            let new = (old + amount / 10.0).max(0.01);
            let ratio = new / old;
            let origin = self.origin(rect);
            let centre = origin + self.bg_pan;
            let a = anchor.unwrap_or(centre);
            let new_centre = a + (centre - a) * ratio;
            self.bg_pan = new_centre - origin;
            self.bg_zoom = new;
        }
    }

    /// Forget the transform handles (they are rebuilt from the shape the next time they are needed).
    pub fn reset_transform(&mut self) {
        self.transform_quad = None;
        self.transform_entry = None;
        self.rotate_start_logical = None;
        self.rotate_center_radius = None;
    }

    /// Ctrl = add to the selection, Alt = remove from it, otherwise start a new selection.
    pub fn select_mode(ctrl: bool, alt: bool) -> SelectMode {
        if ctrl && !alt {
            SelectMode::Add
        } else if alt && !ctrl {
            SelectMode::Remove
        } else {
            SelectMode::New
        }
    }

    /// Screen position of a point.
    pub fn point_screen(p: &Point, origin: Pos2, scale: f32) -> Pos2 {
        Pos2::new(origin.x + p.x as f32 * scale, origin.y + p.y as f32 * scale)
    }

    /// Find the point under a screen position. Control points are checked first, then main points,
    /// which win where they overlap a handle (as in the original).
    pub fn find_point_at(drawing: &Drawing, pos: Pos2, origin: Pos2, scale: f32) -> Option<Id> {
        let mut best_ctrl: Option<(Id, f32)> = None;
        let mut best_main: Option<(Id, f32)> = None;
        for (id, p) in &drawing.points {
            let d = Self::point_screen(p, origin, scale).distance(pos);
            if d > POINT_HIT_RADIUS {
                continue;
            }
            let slot = match p.kind {
                PointKind::Control => &mut best_ctrl,
                PointKind::Main => &mut best_main,
            };
            // Ties are broken by id so hovering is stable between frames.
            if slot.is_none_or(|(bid, bd)| d < bd || (d == bd && *id < bid)) {
                *slot = Some((*id, d));
            }
        }
        best_main.or(best_ctrl).map(|(id, _)| id)
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
            SelectMode::Remove => self.selected.retain(|&x| x != id),
        }
    }

    /// Select all points inside a screen-space rectangle (SelectPointsWithin).
    pub fn select_points_in_rect(
        &mut self,
        drawing: &Drawing,
        origin: Pos2,
        scale: f32,
        a: Pos2,
        b: Pos2,
        mode: SelectMode,
    ) {
        let rect = egui::Rect::from_two_pos(a, b);
        let mut inside: Vec<Id> = Vec::new();
        for (id, p) in &drawing.points {
            if rect.contains(Self::point_screen(p, origin, scale)) {
                inside.push(*id);
            }
        }
        match mode {
            SelectMode::New => self.selected = inside,
            SelectMode::Add => {
                for id in inside {
                    if !self.selected.contains(&id) {
                        self.selected.push(id);
                    }
                }
            }
            SelectMode::Remove => self.selected.retain(|id| !inside.contains(id)),
        }
    }

    /// Rebuild the transform handles from the current shape. Returns false when the shape cannot be
    /// transformed (fewer than two commands or smaller than 5 px, as in the original).
    pub fn init_transform(&mut self, drawing: &Drawing, scale: f32) -> bool {
        self.reset_transform();
        if drawing.commands.len() < 2 {
            return false;
        }
        let Some((x1, y1, x2, y2)) = drawing.shape_bounds() else { return false };
        if (x2 - x1) * f64::from(scale) < 5.0 || (y2 - y1) * f64::from(scale) < 5.0 {
            return false;
        }
        let quad = [(x1, y1), (x2, y1), (x2, y2), (x1, y2)];
        self.transform_quad = Some(quad);
        self.transform_entry = Some((drawing.points.clone(), quad));
        true
    }

    /// Points in the order the Tab key cycles through them (controls, then the main point).
    pub fn tab_order(drawing: &Drawing) -> Vec<Id> {
        let mut order = Vec::new();
        for cmd in &drawing.commands {
            order.extend(cmd.all_points());
        }
        order
    }
}

/// Result of processing input for one frame.
#[derive(Debug, Default)]
pub struct MouseFrameResult {
    pub status_coords: (i32, i32),
    pub status_cmd: String,
    /// Sub-shape whose colour dialog should open (double-click in Coloring mode).
    pub request_color_dialog: Option<usize>,
    pub cmd_changed: bool,
    /// Tool picked from the canvas context menu.
    pub set_tool: Option<crate::ui::Tool>,
    /// Translation key of a message for the status bar.
    pub message: Option<&'static str>,
}
