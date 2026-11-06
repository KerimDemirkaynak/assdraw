use super::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointKind {
    Main,
    Control,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub id: Id,
    pub x: f64,
    pub y: f64,
    pub kind: PointKind,
    #[serde(default)]
    pub selected: bool,
}

impl Point {
    pub fn dist_to_xy(&self, x: f64, y: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }
}
