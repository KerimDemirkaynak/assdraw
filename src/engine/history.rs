use super::{Color, DrawCmd, Id, Point, ShapeStyle};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DrawingSnapshot {
    pub points: HashMap<Id, Point>,
    pub commands: Vec<DrawCmd>,
    pub next_id: Id,
    pub shape_colors: HashMap<usize, Color>,
    pub shape_styles: HashMap<usize, ShapeStyle>,
    pub desc: String,
}

#[derive(Debug, Clone)]
pub struct History {
    undo_stack: Vec<DrawingSnapshot>,
    redo_stack: Vec<DrawingSnapshot>,
    max_depth: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: 128,
        }
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn push(&mut self, s: DrawingSnapshot) {
        self.undo_stack.push(s);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Drop the newest undo snapshot (used when a gesture turned out to change nothing).
    pub fn discard_last(&mut self) {
        self.undo_stack.pop();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn top_undo_desc(&self) -> Option<&str> {
        self.undo_stack.last().map(|s| s.desc.as_str())
    }

    pub fn top_redo_desc(&self) -> Option<&str> {
        self.redo_stack.last().map(|s| s.desc.as_str())
    }

    pub fn undo(&mut self, current: DrawingSnapshot) -> Option<DrawingSnapshot> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current);
        Some(prev)
    }

    pub fn redo(&mut self, current: DrawingSnapshot) -> Option<DrawingSnapshot> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current);
        Some(next)
    }
}
