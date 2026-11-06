use assdraw_rs::engine::parse::{parse_ass_into, parse_ass_replace};
use assdraw_rs::engine::Drawing;

#[test]
fn greedy_l_matches_original() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 0 0 l 10 0 10 10 0 10").unwrap();
    assert_eq!(d.commands.len(), 4);
    let s = d.to_ass_string();
    assert!(s.starts_with("m 0 0"));
    assert!(s.contains("l 10 0"));
    assert!(s.contains("l 10 10"));
    assert!(s.contains("l 0 10"));
}

#[test]
fn greedy_b() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 0 0 b 0 10 10 10 10 0 20 0 20 10 30 0").unwrap();
    // m + 2 bezier
    assert_eq!(d.commands.len(), 3);
}

#[test]
fn replace_clears() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 1 1 l 2 2").unwrap();
    parse_ass_replace(&mut d, "m 0 0").unwrap();
    assert_eq!(d.commands.len(), 1);
}

#[test]
fn scale_rotate_preserves_count() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 0 0 l 10 0 l 10 10 l 0 10").unwrap();
    let n = d.points.len();
    d.apply_scale_rotate(2.0, 90.0);
    assert_eq!(d.points.len(), n);
}

#[test]
fn bounds_of_rect() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 0 0 l 100 0 l 100 50 l 0 50").unwrap();
    let (a, b, c, e) = d.bounds().unwrap();
    assert!((a - 0.0).abs() < 1e-6);
    assert!((c - 100.0).abs() < 1e-6);
    assert!((e - 50.0).abs() < 1e-6);
    let _ = b;
}
