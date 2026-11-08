//! ASS drawing command parser — behavior aligned with ASSDrawEngine::ParseASS.
//! Strips ASS override tags (`{\...}`), greedy L/B, S with trailing points, P extends S, C closes.

use super::{Drawing, ShapeStyle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    InvalidNumber(String),
    IncompleteCommand(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty input"),
            ParseError::InvalidNumber(s) => write!(f, "invalid number: {s}"),
            ParseError::IncompleteCommand(s) => write!(f, "incomplete command: {s}"),
        }
    }
}

fn parse_int(tok: &str) -> Option<i64> {
    if let Ok(v) = tok.parse::<i64>() {
        return Some(v);
    }
    // Decimals (as written by Aegisub) are rounded; words such as "inf"/"nan" are not numbers.
    if !tok.is_empty() && tok.chars().all(|c| c.is_ascii_digit() || "+-.eE".contains(c)) {
        if let Ok(f) = tok.parse::<f64>() {
            if f.is_finite() && f.abs() < 1.0e9 {
                return Some(f.round() as i64);
            }
        }
    }
    None
}

/// A spline being collected: all points (the last one is the main point) and the `c` flag.
struct PendingSpline {
    pts: Vec<(i64, i64)>,
    closed: bool,
}

/// Parse ASS drawing commands and **append** them to `drawing` (S/P/C are parsed).
pub fn parse_ass_into(drawing: &mut Drawing, text: &str) -> Result<usize, ParseError> {
    parse_ass_into_opts(drawing, text, true)
}

/// Replace every `{...}` override block with a space, so glued text such as `10{p0}` still
/// yields a clean number token. An unterminated `{` drops the rest of the text.
fn strip_override_blocks(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '{' => {
                depth += 1;
                if depth == 1 {
                    out.push(' ');
                }
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    out.push(' ');
                }
            }
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Port of `ASSDrawEngine::ParseASS`: a tolerant token state machine. Override blocks and unknown
/// words are ignored, `l` and `b` are greedy, incomplete commands are dropped, and `s`/`p`/`c` are
/// only recognised when `parse_spc` is set (the "Parse S/P/C" setting). Returns the number of
/// commands in the drawing afterwards.
pub fn parse_ass_into_opts(
    drawing: &mut Drawing,
    text: &str,
    parse_spc: bool,
) -> Result<usize, ParseError> {
    let mut norm = strip_override_blocks(&text.replace(['\t', '\r', '\n'], " "));
    norm = norm.to_lowercase();
    norm.push_str(" _ _");

    let is_cmd = |t: &str| -> bool {
        matches!(t, "m" | "n" | "l" | "b" | "_") || (parse_spc && matches!(t, "s" | "p" | "c"))
    };

    // The first command of a drawing always becomes an `m` (as in the original AppendCmd).
    let append = |d: &mut Drawing, kind: char, x: i64, y: i64| {
        let kind = if d.commands.is_empty() && kind == 'l' { 'm' } else { kind };
        d.place_cmd(kind, x as f64, y as f64, None);
    };

    let mut currcmd = String::new();
    let mut vals: Vec<i64> = Vec::new();
    let mut pending: Option<PendingSpline> = None;

    for token in norm.split_whitespace() {
        if is_cmd(token) {
            loop {
                let mut done = true;

                if let Some(sp) = pending.take() {
                    let mut sp = sp;
                    let mut ends = true;
                    if currcmd == "p" && vals.len() >= 2 {
                        sp.pts.push((vals[0], vals[1]));
                        ends = false;
                    } else if currcmd == "c" {
                        sp.closed = true;
                    }
                    if ends {
                        let coords: Vec<(f64, f64)> =
                            sp.pts.iter().map(|&(x, y)| (x as f64, y as f64)).collect();
                        if drawing.commands.is_empty() {
                            if let Some(&(x, y)) = sp.pts.last() {
                                append(drawing, 'm', x, y);
                            }
                        } else {
                            drawing.cmd_s(&coords);
                            if sp.closed {
                                drawing.cmd_c();
                            }
                        }
                    } else {
                        pending = Some(sp);
                    }
                }

                match currcmd.as_str() {
                    "m" | "n" if vals.len() >= 2 => {
                        let k = if currcmd == "m" { 'm' } else { 'n' };
                        append(drawing, k, vals[0], vals[1]);
                    }
                    "l" if vals.len() >= 2 => {
                        append(drawing, 'l', vals[0], vals[1]);
                        vals.drain(0..2);
                        if vals.len() >= 2 {
                            done = false;
                        }
                    }
                    "b" if vals.len() >= 6 => {
                        if drawing.commands.is_empty() {
                            append(drawing, 'm', vals[4], vals[5]);
                        } else {
                            drawing.cmd_b(
                                vals[0] as f64,
                                vals[1] as f64,
                                vals[2] as f64,
                                vals[3] as f64,
                                vals[4] as f64,
                                vals[5] as f64,
                            );
                        }
                        vals.drain(0..6);
                        if vals.len() >= 6 {
                            done = false;
                        }
                    }
                    "s" if vals.len() >= 6 => {
                        let num = (vals.len() / 2) * 2;
                        let pts: Vec<(i64, i64)> =
                            (0..num / 2).map(|i| (vals[2 * i], vals[2 * i + 1])).collect();
                        pending = Some(PendingSpline { pts, closed: false });
                    }
                    _ => {}
                }

                if done {
                    break;
                }
            }
            vals.clear();
            currcmd = token.to_string();
        } else if let Some(v) = parse_int(token) {
            vals.push(v);
        }
    }

    Ok(drawing.commands.len())
}

/// True if `text` contains at least one drawing command letter.
fn has_drawing(text: &str) -> bool {
    let plain = text.replace('}', "} ");
    plain.split_whitespace().any(|t| {
        t.len() == 1 && "mnlbspc".contains(t.to_ascii_lowercase().as_str())
    })
}

struct StyledRun {
    style: ShapeStyle,
    text: String,
    overridden: bool,
}

/// Split `text` into drawing runs, each with the style active after the `{...}` blocks before it.
fn split_styled_runs(text: &str, initial: &ShapeStyle) -> Vec<StyledRun> {
    let mut runs: Vec<StyledRun> = Vec::new();
    let mut active = *initial;
    let mut pending = false;
    let mut rest = text;

    let push = |runs: &mut Vec<StyledRun>, seg: &str, style: &ShapeStyle, overridden: bool| -> bool {
        if has_drawing(seg) {
            runs.push(StyledRun { style: *style, text: seg.to_string(), overridden });
            true
        } else {
            false
        }
    };

    loop {
        let Some(open) = rest.find('{') else {
            push(&mut runs, rest, &active, pending);
            break;
        };
        if push(&mut runs, &rest[..open], &active, pending) {
            pending = false;
        }
        let after = &rest[open..];
        let Some(close) = after.find('}') else { break };
        let mut candidate = active;
        let overridden = candidate.import_tags(&after[..=close]);
        if overridden {
            active = candidate;
        }
        pending = pending || overridden;
        rest = &after[close + 1..];
    }
    runs
}

/// Like [`parse_ass_into`], but also reads `\1c \1a \3c \3a \bord` override tags (original
/// `ImportStyledAss`). `style` is the style before parsing and is replaced by the style of the first
/// run. Runs with a different fill colour become per-contour colours. When several runs were all
/// produced by ASSDraw (each has an override block) their x offsets are un-compensated.
pub fn parse_styled_into(
    drawing: &mut Drawing,
    text: &str,
    style: &mut ShapeStyle,
) -> Result<usize, ParseError> {
    parse_styled_into_opts(drawing, text, style, true)
}

/// [`parse_styled_into`] with an explicit "Parse S/P/C" setting.
pub fn parse_styled_into_opts(
    drawing: &mut Drawing,
    text: &str,
    style: &mut ShapeStyle,
    parse_spc: bool,
) -> Result<usize, ParseError> {
    let runs = split_styled_runs(text, style);
    if runs.is_empty() {
        let before = drawing.commands.len();
        parse_ass_into_opts(drawing, text, parse_spc)?;
        return Ok(drawing.commands.len() - before);
    }

    let decompensate = runs.len() > 1 && runs.iter().all(|r| r.overridden);
    let first_fill = runs[0].style.fill;
    let first_tags = runs[0].style.to_tags();
    let mut total = 0usize;
    let mut cumulative = 0.0_f64;

    for run in &runs {
        let first_id = drawing.next_id;
        let contours_before = drawing.contour_count();
        let cmds_before = drawing.commands.len();
        parse_ass_into_opts(drawing, &run.text, parse_spc)?;
        total += drawing.commands.len() - cmds_before;

        let xs: Vec<f64> = drawing
            .points
            .iter()
            .filter(|(id, _)| **id >= first_id)
            .map(|(_, p)| p.x)
            .collect();
        let lo = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if decompensate && cumulative != 0.0 {
            for (id, p) in drawing.points.iter_mut() {
                if *id >= first_id {
                    p.x += cumulative;
                }
            }
        }
        if !xs.is_empty() {
            cumulative += hi - lo;
        }

        let differs = run.style.to_tags() != first_tags;
        if run.style.fill != first_fill || differs {
            for c in contours_before..drawing.contour_count() {
                drawing.shape_colors.insert(c, run.style.fill);
                if differs {
                    drawing.shape_styles.insert(c, run.style);
                }
            }
        }
    }

    *style = runs[0].style;
    Ok(total)
}

/// Replace the drawing with styled ASS text.
pub fn parse_styled_replace(
    drawing: &mut Drawing,
    text: &str,
    style: &mut ShapeStyle,
    parse_spc: bool,
) -> Result<usize, ParseError> {
    drawing.push_history_desc("Modify drawing commands");
    drawing.clear_shape();
    parse_styled_into_opts(drawing, text, style, parse_spc)
}

/// Replace drawing contents with parsed commands.
pub fn parse_ass_replace(drawing: &mut Drawing, text: &str) -> Result<usize, ParseError> {
    drawing.push_history_desc("Modify drawing commands");
    drawing.points.clear();
    drawing.commands.clear();
    drawing.shape_colors.clear();
    drawing.shape_styles.clear();
    drawing.next_id = 1;
    parse_ass_into(drawing, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{DrawCmd, Drawing};

    fn parse(text: &str, spc: bool) -> Drawing {
        let mut d = Drawing::new();
        parse_ass_into_opts(&mut d, text, spc).unwrap();
        d
    }

    #[test]
    fn strips_override_blocks_even_when_glued() {
        let d = parse("{\\p1}m 0 0 l 10 0 10 10 0 10{\\p0}", true);
        assert_eq!(d.commands.len(), 4);
    }

    #[test]
    fn greedy_lines_and_beziers() {
        assert_eq!(parse("m 0 0 l 10 0 10 10 0 10", true).commands.len(), 4);
        assert_eq!(parse("m 0 0 b 1 1 2 2 3 3 4 4 5 5 6 6", true).commands.len(), 3);
    }

    #[test]
    fn incomplete_commands_are_ignored_not_errors() {
        let d = parse("m 5 5 l 10 10 20", true);
        assert_eq!(d.commands.len(), 2);
        assert_eq!(parse("", true).commands.len(), 0);
        assert_eq!(parse("hello world", true).commands.len(), 0);
    }

    #[test]
    fn first_command_becomes_move() {
        let d = parse("l 3 4 l 5 6", true);
        assert!(matches!(d.commands[0], DrawCmd::M { .. }));
        assert_eq!(d.commands.len(), 2);
    }

    #[test]
    fn spline_with_extension_and_close() {
        let d = parse("m 0 0 s 0 0 10 0 10 10 p 0 10 c", true);
        assert_eq!(d.commands.len(), 3);
        match &d.commands[1] {
            DrawCmd::S { points } => assert_eq!(points.len(), 4),
            other => panic!("{other:?}"),
        }
        assert!(matches!(d.commands[2], DrawCmd::C));
        assert_eq!(d.to_ass_string(), "m 0 0 s 0 0 10 0 10 10 0 10 c");
    }

    #[test]
    fn spc_commands_ignored_when_setting_is_off() {
        // `s` is not a command then, so its numbers feed the greedy `l`.
        let d = parse("m 0 0 l 1 1 s 2 2 3 3 4 4", false);
        assert_eq!(d.commands.len(), 5);
        assert!(d.commands.iter().all(|c| !matches!(c, DrawCmd::S { .. })));
    }

    #[test]
    fn decimals_are_rounded() {
        let d = parse("m 0.4 0.6 l 10.5 3", true);
        assert_eq!(d.to_ass_string(), "m 0 1 l 11 3");
    }
}
