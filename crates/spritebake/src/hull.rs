//! Convex hull via Andrew's monotone chain. Output is CCW with collinear points dropped. The
//! runtime re-hulls these points through parry anyway, so this is just manifest compaction — but a
//! clean hull keeps the on-disk data small and readable.

/// `> 0` if `o -> a -> b` turns counter-clockwise, `< 0` clockwise, `0` collinear.
fn cross(o: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

/// CCW convex hull of `points`. Returns fewer than 3 points only when the input spans no area
/// (empty, single point, or collinear) — the caller treats that as a degenerate sprite.
#[must_use]
pub fn convex_hull(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut pts = points.to_vec();
    pts.sort_by(|a, b| a.partial_cmp(b).expect("finite pixel coords"));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }

    let build = |iter: &mut dyn Iterator<Item = (f32, f32)>| {
        let mut chain: Vec<(f32, f32)> = Vec::new();
        for p in iter {
            while chain.len() >= 2
                && cross(chain[chain.len() - 2], chain[chain.len() - 1], p) <= 0.0
            {
                chain.pop();
            }
            chain.push(p);
        }
        chain.pop(); // last point repeats the next chain's start
        chain
    };

    let mut lower = build(&mut pts.iter().copied());
    let mut upper = build(&mut pts.iter().rev().copied());
    lower.append(&mut upper);
    lower
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_with_interior_point_keeps_four_corners() {
        let hull = convex_hull(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (2.0, 2.0)]);
        assert_eq!(hull.len(), 4);
    }

    #[test]
    fn collinear_collapses_below_three() {
        let hull = convex_hull(&[(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]);
        assert!(hull.len() < 3);
    }

    #[test]
    fn drops_collinear_edge_midpoints() {
        // Triangle with an extra point on the bottom edge: hull is still the 3 corners.
        let hull = convex_hull(&[(0.0, 0.0), (2.0, 0.0), (4.0, 0.0), (2.0, 3.0)]);
        assert_eq!(hull.len(), 3);
    }
}
