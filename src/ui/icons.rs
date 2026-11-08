//! Theme-aware SVG icons.
//!
//! The Font Awesome SVGs in `assets/svg` use `fill="currentColor"`. The SVG rasterizer has no
//! "current colour", so they would always be drawn black (invisible on a dark theme).
//! We therefore rasterize them **white** and let egui multiply them by the text colour
//! (`Image::tint`), which works for both light and dark themes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

macro_rules! svg {
    ($p:literal) => {
        include_str!(concat!("../../assets/svg/", $p))
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Broom,
    Eye,
    Palette,
    ArrowPointer,
    LocationDot,
    Slash,
    BezierCurve,
    Eraser,
    ArrowsRotate,
    DrawPolygon,
    Copy,
    Trash,
    Square,
    SquareCheck,
    Plus,
}

impl Icon {
    fn name(self) -> &'static str {
        match self {
            Icon::Broom => "broom",
            Icon::Eye => "eye",
            Icon::Palette => "palette",
            Icon::ArrowPointer => "arrow-pointer",
            Icon::LocationDot => "location-dot",
            Icon::Slash => "slash",
            Icon::BezierCurve => "bezier-curve",
            Icon::Eraser => "eraser",
            Icon::ArrowsRotate => "arrows-rotate",
            Icon::DrawPolygon => "draw-polygon",
            Icon::Copy => "copy",
            Icon::Trash => "trash-can",
            Icon::Square => "square",
            Icon::SquareCheck => "square-check",
            Icon::Plus => "plus",
        }
    }

    fn raw(self) -> &'static str {
        match self {
            Icon::Broom => svg!("solid/broom.svg"),
            Icon::Eye => svg!("solid/eye.svg"),
            Icon::Palette => svg!("solid/palette.svg"),
            Icon::ArrowPointer => svg!("solid/arrow-pointer.svg"),
            Icon::LocationDot => svg!("solid/location-dot.svg"),
            Icon::Slash => svg!("solid/slash.svg"),
            Icon::BezierCurve => svg!("solid/bezier-curve.svg"),
            Icon::Eraser => svg!("solid/eraser.svg"),
            Icon::ArrowsRotate => svg!("solid/arrows-rotate.svg"),
            Icon::DrawPolygon => svg!("solid/draw-polygon.svg"),
            Icon::Copy => svg!("regular/copy.svg"),
            Icon::Trash => svg!("solid/trash-can.svg"),
            Icon::Square => svg!("regular/square.svg"),
            Icon::SquareCheck => svg!("regular/square-check.svg"),
            Icon::Plus => svg!("solid/plus.svg"),
        }
    }
}

/// Replace every `currentColor` with white and make sure the root element has a white fill.
fn whiten(svg: &str) -> String {
    let s = svg.replace("currentColor", "#ffffff");
    if s.contains("fill=") {
        s
    } else {
        s.replacen("<svg ", "<svg fill=\"#ffffff\" ", 1)
    }
}

fn bytes_for(icon: Icon) -> Arc<[u8]> {
    static CACHE: OnceLock<Mutex<HashMap<Icon, Arc<[u8]>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = match cache.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard
        .entry(icon)
        .or_insert_with(|| Arc::from(whiten(icon.raw()).into_bytes()))
        .clone()
}

/// A white SVG image. Call `.tint(colour)` (or use [`tinted`]) to colour it.
pub fn icon(icon: Icon) -> egui::Image<'static> {
    egui::Image::new(egui::ImageSource::Bytes {
        uri: format!("bytes://icon/{}.svg", icon.name()).into(),
        bytes: egui::load::Bytes::Shared(bytes_for(icon)),
    })
}

/// Icon sized `size` px and coloured to match the current theme's text colour.
pub fn themed(ui: &egui::Ui, which: Icon, size: f32) -> egui::Image<'static> {
    icon(which)
        .fit_to_exact_size(egui::vec2(size, size))
        .tint(ui.visuals().text_color())
}
