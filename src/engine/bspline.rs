//! Uniform cubic B-spline flattener — mirrors AGG `conv_bspline` + interpolation_step.
//! Shared by canvas fill/outline and engine hole/winding polygon approximation.

/// Sample an open uniform cubic B-spline through the given control points.
/// `step` is the parameter increment per sample (~0.05–0.1 is a good balance).
pub fn flatten_bspline(points: &[(f64, f64)], step: f64) -> Vec<(f64, f64)> {
    if points.len() < 2 {
        return points.to_vec();
    }
    if points.len() == 2 {
        return points.to_vec();
    }
    // Pad ends so open curves still produce a full set of cubic segments (AGG-style).
    let mut pts: Vec<(f64, f64)> = Vec::with_capacity(points.len() + 2);
    pts.push(points[0]);
    pts.extend_from_slice(points);
    pts.push(*points.last().unwrap());
    let n = pts.len();
    if n < 4 {
        return points.to_vec();
    }
    let samples_per_seg = ((1.0 / step).ceil() as usize).max(4);
    let mut out = Vec::with_capacity((n - 3) * samples_per_seg + 1);
    out.push(pts[1]); // start near first real control
    for seg in 0..(n - 3) {
        let p0 = pts[seg];
        let p1 = pts[seg + 1];
        let p2 = pts[seg + 2];
        let p3 = pts[seg + 3];
        for s in 1..=samples_per_seg {
            let t = s as f64 / samples_per_seg as f64;
            out.push(cubic_bspline(p0, p1, p2, p3, t));
        }
    }
    out
}

fn cubic_bspline(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let t2 = t * t;
    let t3 = t2 * t;
    let b0 = (-t3 + 3.0 * t2 - 3.0 * t + 1.0) / 6.0;
    let b1 = (3.0 * t3 - 6.0 * t2 + 4.0) / 6.0;
    let b2 = (-3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0) / 6.0;
    let b3 = t3 / 6.0;
    (
        b0 * p0.0 + b1 * p1.0 + b2 * p2.0 + b3 * p3.0,
        b0 * p0.1 + b1 * p1.1 + b2 * p2.1 + b3 * p3.1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bspline_two_points_is_line() {
        let pts = [(0.0, 0.0), (10.0, 0.0)];
        let flat = flatten_bspline(&pts, 0.1);
        assert_eq!(flat.len(), 2);
    }

    #[test]
    fn bspline_produces_samples() {
        let pts = [(0.0, 0.0), (10.0, 20.0), (20.0, 0.0), (30.0, 10.0)];
        let flat = flatten_bspline(&pts, 0.1);
        assert!(flat.len() > 4);
    }
}
