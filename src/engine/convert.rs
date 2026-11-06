//! Command conversion utilities (mirrors original context menu: line↔bezier, C1 continuous Bezier, move to 00).

use super::{DrawCmd, Drawing, Id, PointKind};

impl Drawing {
    /// Convert consecutive L commands into B with auto controls (approx).
    pub fn convert_lines_to_bezier(&mut self, smoothness: f64) {
        self.push_history();
        let old = std::mem::take(&mut self.commands);
        let mut new_cmds = Vec::new();
        let mut prev_main: Option<(f64, f64)> = None;
        for cmd in &old {
            match cmd {
                DrawCmd::L { point } => {
                    if let Some(p) = self.points.get(point) {
                        if let Some((px, py)) = prev_main {
                            let (x, y) = (p.x, p.y);
                            let dx = x - px;
                            let dy = y - py;
                            let len = (dx * dx + dy * dy).sqrt().max(1.0);
                            let nx = -dy / len * smoothness * 20.0;
                            let ny = dx / len * smoothness * 20.0;
                            let c1 = self.add_point(px + dx * 0.33 + nx, py + dy * 0.33 + ny, PointKind::Control);
                            let c2 = self.add_point(px + dx * 0.66 - nx, py + dy * 0.66 - ny, PointKind::Control);
                            new_cmds.push(DrawCmd::B {
                                control1: c1,
                                control2: c2,
                                end: *point,
                                c1_cont: false,
                            });
                            prev_main = Some((x, y));
                        } else {
                            new_cmds.push(cmd.clone());
                            prev_main = Some((p.x, p.y));
                        }
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

    /// Convert B commands to L (drop controls, keep end points).
    pub fn convert_bezier_to_lines(&mut self) {
        self.push_history();
        let old = std::mem::take(&mut self.commands);
        self.commands = old
            .into_iter()
            .map(|cmd| match cmd {
                DrawCmd::B { end, .. } => DrawCmd::L { point: end },
                other => other,
            })
            .collect();
    }

    /// Toggle C1 continuity (collinear smooth control handles) for a Bezier command.
    pub fn toggle_c1_continuity(&mut self, point_id: Id) -> bool {
        // Find which command owns or precedes this point
        for cmd in &mut self.commands {
            if let DrawCmd::B { control1, control2, end, ref mut c1_cont } = cmd {
                if *control1 == point_id || *control2 == point_id || *end == point_id {
                    *c1_cont = !*c1_cont;
                    return *c1_cont;
                }
            }
        }
        false
    }

    /// Align control handles across adjacent Bezier commands when C1 continuity is active.
    pub fn enforce_c1_continuity(&mut self, moved_ctrl_id: Id) {
        // If moved_ctrl_id is control2 of cmd(i), align control1 of cmd(i+1) across end(i).
        let len = self.commands.len();
        for i in 0..len {
            if let DrawCmd::B { control2, end, c1_cont: true, .. } = self.commands[i] {
                if control2 == moved_ctrl_id && i + 1 < len {
                    if let DrawCmd::B { control1: next_c1, .. } = self.commands[i + 1] {
                        if let (Some(c2), Some(e), Some(c1)) = (
                            self.points.get(&control2).copied(),
                            self.points.get(&end).copied(),
                            self.points.get(&next_c1).copied(),
                        ) {
                            // Vector from end to c2
                            let dx = c2.x - e.x;
                            let dy = c2.y - e.y;
                            let dist = (c1.x - e.x).hypot(c1.y - e.y).max(1.0);
                            let len_c2 = dx.hypot(dy).max(1.0);
                            let unit_x = -dx / len_c2;
                            let unit_y = -dy / len_c2;
                            if let Some(target) = self.points.get_mut(&next_c1) {
                                target.x = e.x + unit_x * dist;
                                target.y = e.y + unit_y * dist;
                            }
                        }
                    }
                }
            }
        }
    }
}
