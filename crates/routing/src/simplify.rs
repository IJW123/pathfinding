use bevy::math::Vec2;

/// Ramer–Douglas–Peucker: drop points whose perpendicular deviation from the running chord is under
/// `tolerance`. Endpoints are always kept. Thins the dense, staircased A* output into a lean
/// polyline before it reaches the spline smoother.
#[must_use]
pub fn simplify(points: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    rdp(points, 0, points.len() - 1, tolerance, &mut keep);

    points
        .iter()
        .zip(keep)
        .filter_map(|(p, k)| k.then_some(*p))
        .collect()
}

/// Mark the point of maximum deviation on `[first, last]` for keeping, then recurse on each half.
fn rdp(points: &[Vec2], first: usize, last: usize, tolerance: f32, keep: &mut [bool]) {
    let (mut farthest, mut max_dist) = (first, 0.0);
    for (i, p) in points.iter().enumerate().take(last).skip(first + 1) {
        let d = perp_distance(*p, points[first], points[last]);
        if d > max_dist {
            farthest = i;
            max_dist = d;
        }
    }

    if max_dist > tolerance {
        keep[farthest] = true;
        rdp(points, first, farthest, tolerance, keep);
        rdp(points, farthest, last, tolerance, keep);
    }
}

/// Perpendicular distance from `p` to the line through `a`–`b` (point-to-point distance if `a == b`).
fn perp_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len = ab.length();
    if len > 0.0 {
        (ab.perp_dot(p - a) / len).abs()
    } else {
        p.distance(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_collinear_run_to_endpoints() {
        let line: Vec<Vec2> = (0..=10).map(|i| Vec2::new(i as f32, 0.0)).collect();
        let out = simplify(&line, 1.0);
        assert_eq!(out, vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)]);
    }

    #[test]
    fn flattens_staircase_into_diagonal() {
        // Single-cell staircase climbing a diagonal: deviation from the chord is <= step/2 = 0.5,
        // so a one-step (1.0) tolerance flattens it to the two corners.
        let stair = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(2.0, 2.0),
        ];
        let out = simplify(&stair, 1.0);
        assert_eq!(out, vec![Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0)]);
    }

    #[test]
    fn preserves_a_genuine_bend() {
        let bend = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(50.0, 50.0),
        ];
        let out = simplify(&bend, 1.0);
        assert_eq!(out, bend);
    }
}
