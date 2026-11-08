//! Shape style (fill / outline) and the ASS override tags that carry it.
//! Mirrors the original `ShapeStyle` (shape_style.cpp): `\p1\bord\1c\1a\3c\3a`.

use super::Color;
use serde::{Deserialize, Serialize};

/// Opacity uses the UI convention: 255 = opaque, 0 = transparent (ASS alpha is the inverse).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShapeStyle {
    pub fill_enabled: bool,
    pub fill: Color,
    pub fill_opacity: u8,
    pub outline_enabled: bool,
    pub outline_width: f64,
    pub outline: Color,
    pub outline_opacity: u8,
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            fill_enabled: true,
            fill: Color::from_rgb(0, 102, 255),
            fill_opacity: 255,
            outline_enabled: false,
            outline_width: 1.0,
            outline: Color::BLACK,
            outline_opacity: 255,
        }
    }
}

fn alpha_tag(opacity: u8) -> String {
    format!("&H{:02X}&", 255 - opacity)
}

fn color_from_bgr(value: u32, base: Color) -> Color {
    Color {
        r: (value & 0xff) as u8,
        g: ((value >> 8) & 0xff) as u8,
        b: ((value >> 16) & 0xff) as u8,
        a: base.a,
    }
}

/// Read `\<tag>&H<hex>&` with exactly `digits` hex digits (case-insensitive).
fn read_hex(block: &str, tag: &str, digits: usize) -> Option<u32> {
    let lower = block.to_ascii_lowercase();
    let marker = format!("\\{tag}&h");
    let start = lower.find(&marker)? + marker.len();
    let rest = lower.get(start..)?;
    let end = rest.find('&')?;
    if end != digits {
        return None;
    }
    u32::from_str_radix(&rest[..end], 16).ok()
}

fn read_number(block: &str, tag: &str) -> Option<f64> {
    let marker = format!("\\{tag}");
    let start = block.find(&marker)? + marker.len();
    let rest = block.get(start..)?;
    let len = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+'))
        .unwrap_or(rest.len());
    rest[..len].parse::<f64>().ok().filter(|v| v.is_finite())
}

impl ShapeStyle {
    /// `{\p1\bord…\1c…\1a…\3c…\3a…}`
    pub fn to_tags(&self) -> String {
        let bord = if self.outline_enabled && self.outline_width.is_finite() && self.outline_width > 0.0 {
            format!("{:.2}", self.outline_width)
        } else {
            "0".to_string()
        };
        let fill_opacity = if self.fill_enabled { self.fill_opacity } else { 0 };
        let outline_opacity = if self.outline_enabled { self.outline_opacity } else { 0 };
        format!(
            "{{\\p1\\bord{bord}\\1c&H{}&\\1a{}\\3c&H{}&\\3a{}}}",
            self.fill.to_ass_bgr(),
            alpha_tag(fill_opacity),
            self.outline.to_ass_bgr(),
            alpha_tag(outline_opacity)
        )
    }

    /// Apply the tags found in one `{...}` override block. Returns true if anything changed.
    pub fn import_tags(&mut self, block: &str) -> bool {
        let mut changed = false;
        if let Some(v) = read_hex(block, "1c", 6) {
            self.fill = color_from_bgr(v, self.fill);
            changed = true;
        }
        if let Some(v) = read_hex(block, "3c", 6) {
            self.outline = color_from_bgr(v, self.outline);
            changed = true;
        }
        if let Some(a) = read_hex(block, "1a", 2) {
            self.fill_opacity = (255 - a.min(255)) as u8;
            self.fill_enabled = a != 255;
            changed = true;
        }
        if let Some(a) = read_hex(block, "3a", 2) {
            self.outline_opacity = (255 - a.min(255)) as u8;
            changed = true;
        }
        if let Some(w) = read_number(block, "bord") {
            self.outline_width = w.max(0.0);
            self.outline_enabled = self.outline_width > 0.0 && self.outline_opacity > 0;
            changed = true;
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tags() {
        assert_eq!(
            ShapeStyle::default().to_tags(),
            "{\\p1\\bord0\\1c&HFF6600&\\1a&H00&\\3c&H000000&\\3a&HFF&}"
        );
    }

    #[test]
    fn import_reads_colors_alpha_and_border() {
        let mut s = ShapeStyle::default();
        let changed = s.import_tags("{\\p1\\bord2.00\\1c&H0000FF&\\1a&H80&\\3c&HFF0000&\\3a&H00&}");
        assert!(changed);
        assert_eq!((s.fill.r, s.fill.g, s.fill.b), (255, 0, 0));
        assert_eq!(s.fill_opacity, 127);
        assert!(s.outline_enabled);
        assert!((s.outline_width - 2.0).abs() < 1e-9);
        assert_eq!(s.outline.b, 255);
    }

    #[test]
    fn plain_block_changes_nothing() {
        let mut s = ShapeStyle::default();
        assert!(!s.import_tags("{\\p1}"));
    }
}
