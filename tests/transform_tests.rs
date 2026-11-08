//! Bezier conversion and bilinear deformation behave like ASSDraw3.

use assdraw_rs::engine::{parse_ass_into, DrawCmd, Drawing};

#[test]
fn line_to_bezier_converts_only_that_command_and_keeps_the_shape() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 0 0 l 30 0 l 30 30").unwrap();
    let id = match &d.commands[2] {
        DrawCmd::L { point } => *point,
        other => panic!("unexpected command {other:?}"),
    };
    assert!(d.convert_line_to_bezier_at(id));
    assert!(matches!(d.commands[1], DrawCmd::L { .. }), "first line must stay a line");
    match &d.commands[2] {
        DrawCmd::B { control1, control2, .. } => {
            let (c1, c2) = (d.points[control1], d.points[control2]);
            // controls sit at 1/3 and 2/3 of the straight segment (30,0) -> (30,30)
            assert!((c1.x - 30.0).abs() < 1e-9 && (c1.y - 10.0).abs() < 1e-9);
            assert!((c2.x - 30.0).abs() < 1e-9 && (c2.y - 20.0).abs() < 1e-9);
        }
        other => panic!("expected a Bezier, got {other:?}"),
    }
}

#[test]
fn bezier_to_line_removes_the_control_points() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 0 0 b 10 0 20 0 30 0").unwrap();
    let before = d.points.len();
    let end = match &d.commands[1] {
        DrawCmd::B { end, .. } => *end,
        other => panic!("unexpected command {other:?}"),
    };
    assert!(d.convert_bezier_to_line_at(end));
    assert!(matches!(d.commands[1], DrawCmd::L { .. }));
    assert_eq!(d.points.len(), before - 2);
}

#[test]
fn bilinear_maps_from_the_originals_and_does_not_accumulate() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 0 0 l 10 0 l 10 10 l 0 10").unwrap();
    let original = d.points.clone();
    let rect = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
    let quad = [(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)];

    d.apply_bilinear_from(&original, rect, quad);
    d.apply_bilinear_from(&original, rect, quad); // same input, same result

    let max_x = d.points.values().map(|p| p.x).fold(f64::MIN, f64::max);
    let max_y = d.points.values().map(|p| p.y).fold(f64::MIN, f64::max);
    assert!((max_x - 20.0).abs() < 1e-9 && (max_y - 20.0).abs() < 1e-9);
}
