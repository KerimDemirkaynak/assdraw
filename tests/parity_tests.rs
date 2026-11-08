//! Behaviour that must match ASSDraw3 (canvas.cpp / engine.cpp).

use assdraw_rs::engine::{parse_ass_into, DrawCmd, Drawing, PointKind};

fn drawing(text: &str) -> Drawing {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, text).unwrap();
    d
}

#[test]
fn first_command_cannot_be_deleted_while_others_exist() {
    let mut d = drawing("m 0 0 l 10 0 l 10 10");
    let first = d.commands[0].main_point().unwrap();
    assert!(!d.delete_command_of_point(first));
    assert_eq!(d.commands.len(), 3);
    let last = d.commands[2].main_point().unwrap();
    assert!(d.delete_command_of_point(last));
    assert_eq!(d.commands.len(), 2);
    let second = d.commands[1].main_point().unwrap();
    assert!(d.delete_command_of_point(second));
    assert!(d.delete_command_of_point(first), "the only command may go");
    assert!(d.commands.is_empty());
    assert!(d.points.is_empty());
}

#[test]
fn deleting_a_bezier_control_point_removes_the_whole_command() {
    let mut d = drawing("m 0 0 b 1 1 2 2 3 3 l 5 5");
    let ctrl = match &d.commands[1] {
        DrawCmd::B { control1, .. } => *control1,
        other => panic!("{other:?}"),
    };
    assert!(d.delete_command_of_point(ctrl));
    assert_eq!(d.to_ass_string(), "m 0 0 l 5 5");
}

#[test]
fn clicking_an_existing_point_inserts_after_its_command() {
    let mut d = drawing("m 0 0 l 10 0 l 20 0");
    let (idx, _) = d.place_cmd('l', 5.0, 5.0, Some(0));
    assert_eq!(idx, 1);
    assert_eq!(d.to_ass_string(), "m 0 0 l 5 5 l 10 0 l 20 0");
}

#[test]
fn drawing_l_on_an_empty_canvas_starts_with_a_move() {
    let mut d = Drawing::new();
    d.place_cmd('l', 4.0, 6.0, None);
    assert_eq!(d.to_ass_string(), "m 4 6 l 4 6");
}

#[test]
fn new_bezier_gets_controls_at_thirds() {
    let mut d = drawing("m 0 0");
    d.place_cmd('b', 30.0, 60.0, None);
    assert_eq!(d.to_ass_string(), "m 0 0 b 10 20 20 40 30 60");
}

#[test]
fn spline_points_are_controls_except_the_last() {
    let d = drawing("m 0 0 s 0 0 10 0 10 10 0 10");
    if let DrawCmd::S { points } = &d.commands[1] {
        let kinds: Vec<PointKind> = points.iter().map(|id| d.points[id].kind).collect();
        assert_eq!(kinds, vec![PointKind::Control, PointKind::Control, PointKind::Control, PointKind::Main]);
    } else {
        panic!("expected a spline");
    }
}

#[test]
fn extend_and_close_only_work_after_a_spline() {
    let mut d = drawing("m 0 0 l 5 5");
    assert!(!d.extend_last_spline(1.0, 1.0));
    assert!(!d.close_last_spline());
    let mut d = drawing("m 0 0 s 0 0 10 0 10 10");
    assert!(d.extend_last_spline(0.0, 10.0));
    assert!(d.close_last_spline());
    assert_eq!(d.to_ass_string(), "m 0 0 s 0 0 10 0 10 10 0 10 c");
}

#[test]
fn nested_contours_with_the_same_winding_are_not_holes() {
    // Non-zero winding: an inner square wound the same way stays filled.
    let d = drawing("m 0 0 l 100 0 l 100 100 l 0 100 m 20 20 l 80 20 l 80 80 l 20 80");
    let groups = d.subshape_groups();
    assert_eq!(groups.len(), 2, "{groups:?}");
}

#[test]
fn opposite_winding_makes_a_hole_that_belongs_to_its_parent() {
    let d = drawing("m 0 0 l 100 0 l 100 100 l 0 100 m 20 20 l 20 80 l 80 80 l 80 20");
    let groups = d.subshape_groups();
    assert_eq!(groups.len(), 1, "{groups:?}");
    assert_eq!(groups[0].1, vec![0, 1]);
    assert_eq!(d.subshape_at(10.0, 10.0), Some(0));
    assert_eq!(d.subshape_at(50.0, 50.0), None, "inside the hole");
    assert_eq!(d.subshape_at(500.0, 500.0), None);
}

#[test]
fn shape_bounds_ignore_control_points() {
    let d = drawing("m 0 0 b 0 100 100 100 100 0");
    let (_, _, _, max_y) = d.shape_bounds().unwrap();
    assert!(max_y < 80.0, "curve peaks at 75, controls sit at 100 (got {max_y})");
}

#[test]
fn transforms_keep_integer_coordinates() {
    let mut d = drawing("m 0 0 l 7 0 l 7 7 l 0 7");
    d.apply_scale_rotate(1.5, 33.0);
    assert!(d.points.values().all(|p| p.x == p.x.round() && p.y == p.y.round()));
}

#[test]
fn moving_a_bezier_control_keeps_smooth_connection_when_enabled() {
    let mut d = drawing("m 0 0 b 0 10 10 10 10 0 b 10 -10 20 -10 20 0");
    let end1 = d.commands[1].main_point().unwrap();
    assert!(d.can_toggle_c1(end1));
    d.toggle_c1_continuity(end1);
    let c2 = match &d.commands[1] {
        DrawCmd::B { control2, .. } => *control2,
        _ => unreachable!(),
    };
    d.move_points(&[c2], 0.0, 4.0);
    let next_c1 = match &d.commands[2] {
        DrawCmd::B { control1, .. } => *control1,
        _ => unreachable!(),
    };
    let (e, c, n) = (d.points[&end1], d.points[&c2], d.points[&next_c1]);
    assert_eq!((n.x - e.x, n.y - e.y), (e.x - c.x, e.y - c.y));
}
