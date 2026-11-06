//! Styled ASS serialization — port concepts from styled_ass.hpp/cpp.
//! Contour-aware runs with per-subshape colors.

use crate::engine::{Color, DrawCmd, Drawing, Point};
use std::collections::HashMap;

/// One logical drawing run with a style (fill).
#[derive(Debug, Clone)]
pub struct StyledRun {
    pub fill: Color,
    pub commands_ass: String,
}

/// Build styled runs from a drawing (splits on contour starts).
pub fn build_styled_runs(drawing: &Drawing, default_fill: Color) -> Vec<StyledRun> {
    let mut runs = Vec::new();
    let mut batch: Vec<&DrawCmd> = Vec::new();
    let mut cidx = 0usize;

    let flush = |batch: &mut Vec<&DrawCmd>, cidx: usize, runs: &mut Vec<StyledRun>, drawing: &Drawing| {
        if batch.is_empty() {
            return;
        }
        let fill = drawing.get_contour_color(cidx).copied().unwrap_or(default_fill);
        let commands_ass = cmds_to_string(batch, &drawing.points);
        runs.push(StyledRun { fill, commands_ass });
        batch.clear();
    };

    for cmd in &drawing.commands {
        if cmd.is_contour_start() && !batch.is_empty() {
            flush(&mut batch, cidx, &mut runs, drawing);
            cidx += 1;
        }
        batch.push(cmd);
    }
    flush(&mut batch, cidx, &mut runs, drawing);
    runs
}

fn cmds_to_string(cmds: &[&DrawCmd], points: &HashMap<u64, Point>) -> String {
    let mut out = String::new();
    for cmd in cmds {
        if !out.is_empty() {
            out.push(' ');
        }
        match cmd {
            DrawCmd::M { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("m {} {}", r(p.x), r(p.y)));
                }
            }
            DrawCmd::N { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("n {} {}", r(p.x), r(p.y)));
                }
            }
            DrawCmd::L { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("l {} {}", r(p.x), r(p.y)));
                }
            }
            DrawCmd::B { control1, control2, end, .. } => {
                if let (Some(c1), Some(c2), Some(e)) =
                    (points.get(control1), points.get(control2), points.get(end))
                {
                    out.push_str(&format!(
                        "b {} {} {} {} {} {}",
                        r(c1.x), r(c1.y), r(c2.x), r(c2.y), r(e.x), r(e.y)
                    ));
                }
            }
            DrawCmd::S { points: pts } => {
                out.push('s');
                for pid in pts {
                    if let Some(p) = points.get(pid) {
                        out.push_str(&format!(" {} {}", r(p.x), r(p.y)));
                    }
                }
            }
            DrawCmd::P { point } => {
                if let Some(p) = points.get(point) {
                    out.push_str(&format!("p {} {}", r(p.x), r(p.y)));
                }
            }
            DrawCmd::C => out.push('c'),
        }
    }
    out
}

fn r(v: f64) -> i64 {
    v.round() as i64
}

/// Serialize runs to Aegisub-friendly concatenated override blocks.
pub fn serialize_styled_ass(runs: &[StyledRun], border: Color, border_width: f64) -> String {
    let mut out = String::new();
    for run in runs {
        out.push_str(&format!(
            "{{\\an7\\pos(0,0)\\p1\\bord{}\\1c&H{}&\\3c&H{}&}}{}{{\\p0}}",
            border_width,
            run.fill.to_ass_bgr(),
            border.to_ass_bgr(),
            run.commands_ass
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::parse::parse_ass_into;

    #[test]
    fn two_contours_two_runs() {
        let mut d = Drawing::new();
        parse_ass_into(&mut d, "m 0 0 l 10 0 l 10 10 m 20 20 l 30 20").unwrap();
        d.set_contour_color(0, Color::from_rgb(255, 0, 0));
        d.set_contour_color(1, Color::from_rgb(0, 255, 0));
        let runs = build_styled_runs(&d, Color::WHITE);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].fill.r, 255);
        assert_eq!(runs[1].fill.g, 255);
    }
}
