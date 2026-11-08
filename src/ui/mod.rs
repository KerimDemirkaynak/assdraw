pub mod canvas;
pub mod dialogs;
pub mod icons;
pub mod panels;

pub use canvas::*;
pub use dialogs::*;
pub use panels::*;

use crate::i18n::{t, Language};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    MoveTo,
    MoveNoClose,
    LineTo,
    Bezier,
    Spline,
    ExtendSpline,
    Close,
    Delete,
    ScaleRotate,
    Bilinear,
    ColorPick,
    Pan,
}

impl Tool {
    pub fn label(self, lang: Language) -> &'static str {
        match self {
            Tool::Select => t(lang, "tool_select"),
            Tool::MoveTo => "M",
            Tool::MoveNoClose => "N",
            Tool::LineTo => "L",
            Tool::Bezier => "B",
            Tool::Spline => "S",
            Tool::ExtendSpline => "P",
            Tool::Close => "C",
            Tool::Delete => t(lang, "action_delete"),
            Tool::ScaleRotate => t(lang, "tool_scale_rotate"),
            Tool::Bilinear => t(lang, "tool_bilinear"),
            Tool::ColorPick => t(lang, "tool_coloring"),
            Tool::Pan => t(lang, "tool_pan"),
        }
    }

    pub fn tooltip(self, lang: Language) -> &'static str {
        match self {
            Tool::Select => t(lang, "tool_select"),
            Tool::MoveTo => t(lang, "tool_m"),
            Tool::MoveNoClose => t(lang, "tool_n"),
            Tool::LineTo => t(lang, "tool_l"),
            Tool::Bezier => t(lang, "tool_b"),
            Tool::Spline => t(lang, "tool_s"),
            Tool::ExtendSpline => t(lang, "tool_p"),
            Tool::Close => t(lang, "tool_c"),
            Tool::Delete => t(lang, "tool_del"),
            Tool::ScaleRotate => t(lang, "tool_scale_rotate"),
            Tool::Bilinear => t(lang, "tool_bilinear"),
            Tool::ColorPick => t(lang, "tool_coloring"),
            Tool::Pan => t(lang, "tool_pan"),
        }
    }
}
