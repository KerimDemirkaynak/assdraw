//! Command conversion utilities (mirrors original context menu: line↔bezier, C1 continuous Bezier, move to 00).

use super::{DrawCmd, Drawing, Id, PointKind};

impl Drawing {
    /// Convert consecutive L commands into B with auto controls (approx).
    /// Convert every L command to a Bezier whose controls sit at 1/3 and 2/3 of the line,
    /// so the shape does not change (ASSDraw3's `DrawCmd_B::Init`).
    pub fn convert_lines_to_bezier(&mut self, _smoothness: f64) {
        self.push_history();
        let old = std::mem::take(&mut self.commands);
        let mut new_cmds = Vec::new();
        let mut prev_main: Option<(f64, f64)> = None;
        for cmd in &old {
            match cmd {
                DrawCmd::L { point } => {
                    if let Some(p) = self.points.get(point).copied() {
                        if let Some((px, py)) = prev_main {
                            let (a, b) = Drawing::thirds_controls((px, py), (p.x, p.y));
                            let c1 = self.add_point(a.0, a.1, PointKind::Control);
                            let c2 = self.add_point(b.0, b.1, PointKind::Control);
                            new_cmds.push(DrawCmd::B {
                                control1: c1,
                                control2: c2,
                                end: *point,
                                c1_cont: false,
                            });
                        } else {
                            new_cmds.push(cmd.clone());
                        }
                        prev_main = Some((p.x, p.y));
                    }
                }
                other => {
                    new_cmds.push(other.clone());
                    if let Some(id) = other.main_point() {
                        if let Some(p) = self.points.get(&id) {
                            prev_main = Some((p.x, p.y));
                        }
                    }
                }
            }
        }
        self.commands = new_cmds;
    }

    /// Convert only the L command that owns `id` (right-click on a point). Returns true if done.
    pub fn convert_line_to_bezier_at(&mut self, id: Id) -> bool {
        let Some(idx) = self.commands.iter().position(|c| c.all_points().contains(&id)) else {
            return false;
        };
        let DrawCmd::L { point } = &self.commands[idx] else {
            return false;
        };
        let point = *point;
        let Some(prev) = idx.checked_sub(1).and_then(|i| self.commands[i].main_point()) else {
            return false;
        };
        let (Some(p0), Some(p1)) = (self.points.get(&prev).copied(), self.points.get(&point).copied()) else {
            return false;
        };
        self.push_history_desc("Convert Line to Bezier");
        let (a, b) = Drawing::thirds_controls((p0.x, p0.y), (p1.x, p1.y));
        let c1 = self.add_point(a.0, a.1, PointKind::Control);
        let c2 = self.add_point(b.0, b.1, PointKind::Control);
        self.commands[idx] = DrawCmd::B { control1: c1, control2: c2, end: point, c1_cont: false };
        true
    }

    /// Convert only the B command that owns `id` to a line. Returns true if done.
    pub fn convert_bezier_to_line_at(&mut self, id: Id) -> bool {
        let Some(idx) = self.commands.iter().position(|c| c.all_points().contains(&id)) else {
            return false;
        };
        let DrawCmd::B { control1, control2, end, .. } = &self.commands[idx] else {
            return false;
        };
        let (c1, c2, end) = (*control1, *control2, *end);
        if end != id {
            return false;
        }
        self.push_history_desc("Convert Bezier to Line");
        self.points.remove(&c1);
        self.points.remove(&c2);
        self.commands[idx] = DrawCmd::L { point: end };
        true
    }

    /// Convert B commands to L (drop controls, keep end points).
    pub fn convert_bezier_to_lines(&mut self) {
        self.push_history();
        let old = std::mem::take(&mut self.commands);
        let mut new_cmds = Vec::with_capacity(old.len());
        for cmd in old {
            match cmd {
                DrawCmd::B { control1, control2, end, .. } => {
                    self.points.remove(&control1);
                    self.points.remove(&control2);
                    new_cmds.push(DrawCmd::L { point: end });
                }
                other => new_cmds.push(other),
            }
        }
        self.commands = new_cmds;
    }

    /// True when the point is the main point of a Bezier that is followed by another Bezier
    /// (the only place where the "smooth connection" option makes sense).
    pub fn can_toggle_c1(&self, point_id: Id) -> bool {
        self.commands.windows(2).any(|w| {
            matches!(&w[0], DrawCmd::B { end, .. } if *end == point_id)
                && matches!(w[1], DrawCmd::B { .. })
        })
    }

    /// Toggle C1 continuity (collinear smooth control handles) for the Bezier ending at `point_id`.
    pub fn toggle_c1_continuity(&mut self, point_id: Id) -> bool {
        for cmd in &mut self.commands {
            if let DrawCmd::B { end, ref mut c1_cont, .. } = cmd {
                if *end == point_id {
                    *c1_cont = !*c1_cont;
                    return *c1_cont;
                }
            }
        }
        false
    }

    /// Keep C1 continuity: the handle on the other side of a shared main point is the mirror
    /// image of the moved one (ASSDraw3's `EnforceC1Continuity`).
    pub fn enforce_c1_continuity(&mut self, moved_ctrl_id: Id) {
        let len = self.commands.len();
        for i in 0..len {
            // moved control2 of cmd i  →  control1 of cmd i+1 (if cmd i is flagged)
            if let DrawCmd::B { control2, end, c1_cont: true, .. } = self.commands[i] {
                if control2 == moved_ctrl_id && i + 1 < len {
                    if let DrawCmd::B { control1: next_c1, .. } = self.commands[i + 1] {
                        self.mirror_handle(moved_ctrl_id, end, next_c1);
                    }
                }
            }
            // moved control1 of cmd i  →  control2 of cmd i-1 (if cmd i-1 is flagged)
            if i > 0 {
                if let DrawCmd::B { control1, .. } = self.commands[i] {
                    if control1 == moved_ctrl_id {
                        if let DrawCmd::B { control2: prev_c2, end: prev_end, c1_cont: true, .. } =
                            self.commands[i - 1]
                        {
                            self.mirror_handle(moved_ctrl_id, prev_end, prev_c2);
                        }
                    }
                }
            }
        }
    }

    /// Place `target` at the mirror image of `moved` around `pivot`.
    fn mirror_handle(&mut self, moved: Id, pivot: Id, target: Id) {
        if let (Some(m), Some(c)) = (self.points.get(&moved).copied(), self.points.get(&pivot).copied()) {
            if let Some(t) = self.points.get_mut(&target) {
                t.x = (c.x + (c.x - m.x)).round();
                t.y = (c.y + (c.y - m.y)).round();
            }
        }
    }
}
