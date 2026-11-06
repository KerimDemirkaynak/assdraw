//! ASS drawing command parser — behavior aligned with ASSDrawEngine::ParseASS.
//! Strips ASS override tags (`{\...}`), greedy L/B, S with trailing points, P extends S, C closes.

use super::Drawing;

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

/// Strip ASS override tags like `{\p1}`, `{\pos(10,20)}`, etc.
fn strip_ass_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        if ch == '{' {
            in_tag = true;
        } else if ch == '}' {
            in_tag = false;
            out.push(' ');
        } else if !in_tag {
            out.push(ch);
        }
    }
    out
}

/// Tokenize like wxStringTokenizer on whitespace; letters are command heads.
fn tokenize(text: &str) -> Vec<String> {
    let stripped = strip_ass_tags(text);
    stripped
        .replace(['\n', '\r', '\t'], " ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

fn num(s: &str) -> Result<f64, ParseError> {
    s.parse().map_err(|_| ParseError::InvalidNumber(s.to_string()))
}

/// Parse ASS drawing commands and **append** them to `drawing`.
/// Returns number of top-level commands appended.
pub fn parse_ass_into(drawing: &mut Drawing, text: &str) -> Result<usize, ParseError> {
    let tokens = tokenize(text);
    if tokens.is_empty() {
        return Err(ParseError::Empty);
    }

    let mut i = 0usize;
    let mut added = 0usize;
    let mut pending_s: Option<Vec<(f64, f64)>> = None;

    let flush_s = |drawing: &mut Drawing, pending: &mut Option<Vec<(f64, f64)>>, added: &mut usize| {
        if let Some(pts) = pending.take() {
            if pts.len() >= 2 {
                drawing.cmd_s(&pts);
                *added += 1;
            }
        }
    };

    while i < tokens.len() {
        let t = tokens[i].to_lowercase();
        if t.len() == 1 && t.chars().next().unwrap().is_ascii_alphabetic() {
            match t.as_str() {
                "p" | "c" if pending_s.is_some() => {}
                _ => flush_s(drawing, &mut pending_s, &mut added),
            }

            match t.as_str() {
                "m" => {
                    if i + 2 >= tokens.len() {
                        return Err(ParseError::IncompleteCommand("m".into()));
                    }
                    let x = num(&tokens[i + 1])?;
                    let y = num(&tokens[i + 2])?;
                    drawing.cmd_m(x, y);
                    added += 1;
                    i += 3;
                }
                "n" => {
                    if i + 2 >= tokens.len() {
                        return Err(ParseError::IncompleteCommand("n".into()));
                    }
                    let x = num(&tokens[i + 1])?;
                    let y = num(&tokens[i + 2])?;
                    drawing.cmd_n(x, y);
                    added += 1;
                    i += 3;
                }
                "l" => {
                    i += 1;
                    let mut any = false;
                    while i + 1 < tokens.len() {
                        if tokens[i].chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                            break;
                        }
                        let x = match num(&tokens[i]) {
                            Ok(v) => v,
                            Err(_) => break,
                        };
                        let y = match num(&tokens[i + 1]) {
                            Ok(v) => v,
                            Err(_) => break,
                        };
                        drawing.cmd_l(x, y);
                        added += 1;
                        any = true;
                        i += 2;
                    }
                    if !any {
                        return Err(ParseError::IncompleteCommand("l".into()));
                    }
                }
                "b" => {
                    i += 1;
                    let mut any = false;
                    while i + 5 < tokens.len() {
                        if tokens[i].chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                            break;
                        }
                        let Ok(c1x) = num(&tokens[i]) else { break };
                        let Ok(c1y) = num(&tokens[i + 1]) else { break };
                        let Ok(c2x) = num(&tokens[i + 2]) else { break };
                        let Ok(c2y) = num(&tokens[i + 3]) else { break };
                        let Ok(x) = num(&tokens[i + 4]) else { break };
                        let Ok(y) = num(&tokens[i + 5]) else { break };
                        drawing.cmd_b(c1x, c1y, c2x, c2y, x, y);
                        added += 1;
                        any = true;
                        i += 6;
                    }
                    if !any {
                        return Err(ParseError::IncompleteCommand("b".into()));
                    }
                }
                "s" => {
                    i += 1;
                    let mut pts = Vec::new();
                    while i + 1 < tokens.len() {
                        if tokens[i].chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                            break;
                        }
                        let Ok(x) = num(&tokens[i]) else { break };
                        let Ok(y) = num(&tokens[i + 1]) else { break };
                        pts.push((x, y));
                        i += 2;
                    }
                    if pts.len() < 2 {
                        return Err(ParseError::IncompleteCommand("s".into()));
                    }
                    pending_s = Some(pts);
                }
                "p" => {
                    if i + 2 >= tokens.len() {
                        return Err(ParseError::IncompleteCommand("p".into()));
                    }
                    let x = num(&tokens[i + 1])?;
                    let y = num(&tokens[i + 2])?;
                    if let Some(ref mut pts) = pending_s {
                        pts.push((x, y));
                    } else {
                        drawing.cmd_p(x, y);
                        added += 1;
                    }
                    i += 3;
                }
                "c" => {
                    if let Some(pts) = pending_s.take() {
                        if pts.len() >= 2 {
                            drawing.cmd_s(&pts);
                            added += 1;
                        }
                    }
                    drawing.cmd_c();
                    added += 1;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    flush_s(drawing, &mut pending_s, &mut added);
    Ok(added)
}

/// Replace drawing contents with parsed commands.
pub fn parse_ass_replace(drawing: &mut Drawing, text: &str) -> Result<usize, ParseError> {
    drawing.push_history();
    drawing.points.clear();
    drawing.commands.clear();
    drawing.shape_colors.clear();
    drawing.next_id = 1;
    parse_ass_into(drawing, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Drawing;

    #[test]
    fn test_strip_tags() {
        let mut d = Drawing::new();
        parse_ass_into(&mut d, "{\\p1}m 0 0 l 10 0 10 10 0 10{\\p0}").unwrap();
        assert_eq!(d.commands.len(), 4);
    }

    #[test]
    fn greedy_lines() {
        let mut d = Drawing::new();
        parse_ass_into(&mut d, "m 0 0 l 10 0 10 10 0 10").unwrap();
        assert_eq!(d.commands.len(), 4);
    }

    #[test]
    fn greedy_bezier() {
        let mut d = Drawing::new();
        parse_ass_into(&mut d, "m 0 0 b 1 1 2 2 3 3 4 4 5 5 6 6").unwrap();
        assert_eq!(d.commands.len(), 3);
    }
}
