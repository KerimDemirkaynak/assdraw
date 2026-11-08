//! Styled ASS output/import (colour, alpha, border, run compensation).

use assdraw_rs::engine::{parse_ass_into, parse_styled_into, Color, Drawing, ShapeStyle};

#[test]
fn runs_are_compensated_and_round_trip() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 0 0 l 10 0 10 10 m 20 0 l 30 0 30 10").unwrap();
    d.set_contour_color(1, Color::from_rgb(255, 0, 0));

    let line = d.to_styled_ass(&ShapeStyle::default(), false, true);
    assert_eq!(line.matches("{\\p1").count(), 2, "{line}");
    assert!(line.contains("m 0 0 l 10 0 l 10 10"), "{line}");
    // second run is shifted left by the width (10) of the first one
    assert!(line.contains("m 10 0 l 20 0 l 20 10"), "{line}");

    let mut d2 = Drawing::new();
    let mut style = ShapeStyle::default();
    parse_styled_into(&mut d2, &line, &mut style).unwrap();
    assert_eq!(d2.to_ass_string_custom(false), d.to_ass_string_custom(false));
    assert_eq!(d2.get_contour_color(1), Some(&Color::from_rgb(255, 0, 0)));
}

#[test]
fn single_color_is_one_run_and_ignoring_contour_colors_works() {
    let mut d = Drawing::new();
    parse_ass_into(&mut d, "m 0 0 l 10 0 10 10 m 20 0 l 30 0 30 10").unwrap();
    d.set_contour_color(1, Color::from_rgb(255, 0, 0));
    let line = d.to_styled_ass(&ShapeStyle::default(), false, false);
    assert_eq!(line.matches("{\\p1").count(), 1, "{line}");
}

#[test]
fn import_adopts_style_from_tags() {
    let mut d = Drawing::new();
    let mut style = ShapeStyle::default();
    parse_styled_into(
        &mut d,
        "{\\p1\\bord3.00\\1c&H0000FF&\\1a&H00&\\3c&H00FF00&\\3a&H00&}m 0 0 l 5 0 5 5",
        &mut style,
    )
    .unwrap();
    assert_eq!((style.fill.r, style.fill.g, style.fill.b), (255, 0, 0));
    assert!(style.outline_enabled);
    assert!((style.outline_width - 3.0).abs() < 1e-9);
}
