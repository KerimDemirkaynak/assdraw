use super::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrawCmd {
    M { point: Id },
    N { point: Id },
    L { point: Id },
    B {
        control1: Id,
        control2: Id,
        end: Id,
        #[serde(default)]
        c1_cont: bool,
    },
    S { points: Vec<Id> },
    P { point: Id },
    C,
}

impl DrawCmd {
    pub fn main_point(&self) -> Option<Id> {
        match self {
            DrawCmd::M { point } | DrawCmd::N { point } | DrawCmd::L { point } | DrawCmd::P { point } => Some(*point),
            DrawCmd::B { end, .. } => Some(*end),
            DrawCmd::S { points } => points.last().copied(),
            DrawCmd::C => None,
        }
    }

    pub fn all_points(&self) -> Vec<Id> {
        match self {
            DrawCmd::M { point } | DrawCmd::N { point } | DrawCmd::L { point } | DrawCmd::P { point } => vec![*point],
            DrawCmd::B { control1, control2, end, .. } => vec![*control1, *control2, *end],
            DrawCmd::S { points } => points.clone(),
            DrawCmd::C => vec![],
        }
    }

    pub fn is_contour_start(&self) -> bool {
        matches!(self, DrawCmd::M { .. } | DrawCmd::N { .. })
    }
}
