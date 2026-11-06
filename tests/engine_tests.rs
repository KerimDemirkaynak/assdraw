use assdraw_rs::engine::parse::parse_ass_into;
use assdraw_rs::engine::Drawing;

#[test]
fn engine_parse_and_serialize() {
    let mut d = Drawing::new();
    let res = parse_ass_into(&mut d, "m 0 0 l 10 0 10 10 0 10");
    assert!(res.is_ok());
    assert_eq!(d.points.len(), 4);
    assert_eq!(d.commands.len(), 4);

    let ass = d.to_ass_string();
    assert!(ass.starts_with("m 0 0"));
}
