use bevy::prelude::*;

use crate::constants::COLLISION_EPSILON;
use crate::manifold::Manifold;

/// Project a point set onto a unit axis, returning `(min, max)` of the scalar projections.
fn project(points: &[Vec2], axis: Vec2) -> (f32, f32) {
    points
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), &p| {
            let d = p.dot(axis);
            (min.min(d), max.max(d))
        })
}

/// Vertex average — good enough to orient the normal, not a true area centroid.
fn centroid(points: &[Vec2]) -> Vec2 {
    points.iter().copied().fold(Vec2::ZERO, |acc, p| acc + p) / points.len() as f32
}

/// Outward unit normals of each polygon edge (CW perp of the edge vector). Zero-length edges
/// (degenerate input) are skipped.
fn edge_normals(poly: &[Vec2]) -> impl Iterator<Item = Vec2> + '_ {
    let n = poly.len();
    poly.iter().enumerate().filter_map(move |(i, &p)| {
        let edge = poly[(i + 1) % n] - p;
        let normal = Vec2::new(edge.y, -edge.x);
        (normal.length() > COLLISION_EPSILON).then(|| normal.normalize())
    })
}

/// Orient `axis` so it points along `delta` (a→b). When the projection is within epsilon
/// (symmetric/nested overlap — `delta` ⊥ axis), keep the raw axis rather than flipping on noise.
fn orient(axis: Vec2, delta: Vec2) -> Vec2 {
    if axis.dot(delta) < -COLLISION_EPSILON {
        -axis
    } else {
        axis
    }
}

/// SAT between two convex polygons. Normal points from `a` toward `b`, depth is the minimum
/// overlap. `None` if separated or merely grazing (overlap `<= COLLISION_EPSILON`).
#[must_use]
pub fn poly_poly(a: &[Vec2], b: &[Vec2]) -> Option<Manifold> {
    let mut best_depth = f32::INFINITY;
    let mut best_axis = Vec2::ZERO;
    for axis in edge_normals(a).chain(edge_normals(b)) {
        let (a_min, a_max) = project(a, axis);
        let (b_min, b_max) = project(b, axis);
        let overlap = a_max.min(b_max) - a_min.max(b_min);
        if overlap <= COLLISION_EPSILON {
            return None;
        }
        if overlap < best_depth {
            best_depth = overlap;
            best_axis = axis;
        }
    }
    (best_axis != Vec2::ZERO).then(|| Manifold {
        normal: orient(best_axis, centroid(b) - centroid(a)),
        depth: best_depth,
    })
}

/// SAT between a convex polygon (`a`) and a circle (`b`). Tests every edge normal plus the
/// closest-vertex→center axis (circle-vs-corner). Normal points polygon→circle.
#[must_use]
pub fn poly_circle(poly: &[Vec2], c: Vec2, r: f32) -> Option<Manifold> {
    let vertex_axis = poly
        .iter()
        .copied()
        .reduce(|best, p| {
            if p.distance_squared(c) < best.distance_squared(c) {
                p
            } else {
                best
            }
        })
        .and_then(|v| {
            let d = c - v;
            (d.length() > COLLISION_EPSILON).then(|| d.normalize())
        });

    let mut best_depth = f32::INFINITY;
    let mut best_axis = Vec2::ZERO;
    for axis in edge_normals(poly).chain(vertex_axis) {
        let (p_min, p_max) = project(poly, axis);
        let mid = c.dot(axis);
        let overlap = p_max.min(mid + r) - p_min.max(mid - r);
        if overlap <= COLLISION_EPSILON {
            return None;
        }
        if overlap < best_depth {
            best_depth = overlap;
            best_axis = axis;
        }
    }
    (best_axis != Vec2::ZERO).then(|| Manifold {
        normal: orient(best_axis, c - centroid(poly)),
        depth: best_depth,
    })
}

/// Center-distance test. Normal points `a`→`b`; coincident centers fall back to `Vec2::X`.
#[must_use]
pub fn circle_circle(ca: Vec2, ra: f32, cb: Vec2, rb: f32) -> Option<Manifold> {
    let delta = cb - ca;
    let dist = delta.length();
    let depth = (ra + rb) - dist;
    (depth > COLLISION_EPSILON).then(|| Manifold {
        normal: if dist > COLLISION_EPSILON {
            delta / dist
        } else {
            Vec2::X
        },
        depth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(cx: f32, cy: f32, h: f32) -> [Vec2; 4] {
        [
            Vec2::new(cx - h, cy - h),
            Vec2::new(cx + h, cy - h),
            Vec2::new(cx + h, cy + h),
            Vec2::new(cx - h, cy + h),
        ]
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    fn close_vec(a: Vec2, b: Vec2) -> bool {
        close(a.x, b.x) && close(a.y, b.y)
    }

    #[test]
    fn box_box_overlap_normal_and_depth() {
        let a = square(0.0, 0.0, 1.0);
        let b = square(1.5, 0.0, 1.0);
        let m = poly_poly(&a, &b).expect("overlap");
        assert!(close_vec(m.normal, Vec2::X), "normal {:?}", m.normal);
        assert!(close(m.depth, 0.5), "depth {}", m.depth);
    }

    #[test]
    fn box_box_edge_touch_is_none() {
        let a = square(0.0, 0.0, 1.0);
        let b = square(2.0, 0.0, 1.0); // x-ranges touch at x=1, zero overlap
        assert!(poly_poly(&a, &b).is_none());
    }

    #[test]
    fn rotated_overlap_detected_and_oriented() {
        let a = square(0.0, 0.0, 1.0);
        // Diamond (square rotated 45°) centered right of `a`, vertex pointing at it.
        let r = 2.0_f32.sqrt();
        let diamond = [
            Vec2::new(1.9 + r, 0.0),
            Vec2::new(1.9, r),
            Vec2::new(1.9 - r, 0.0),
            Vec2::new(1.9, -r),
        ];
        let m = poly_poly(&a, &diamond).expect("overlap");
        assert!(m.depth > 0.0);
        assert!(m.normal.x > 0.0, "b is to the right, normal {:?}", m.normal);
        assert!(close(m.normal.length(), 1.0));

        let far = [
            Vec2::new(5.0 + r, 0.0),
            Vec2::new(5.0, r),
            Vec2::new(5.0 - r, 0.0),
            Vec2::new(5.0, -r),
        ];
        assert!(poly_poly(&a, &far).is_none());
    }

    #[test]
    fn symmetric_overlap_has_stable_unit_normal() {
        let a = square(0.0, 0.0, 1.0);
        let b = square(0.0, 0.0, 1.0); // concentric — delta ⊥ every axis (guard path)
        let m = poly_poly(&a, &b).expect("overlap");
        assert!(m.normal.is_finite());
        assert!(close(m.normal.length(), 1.0));
        assert!(close(m.depth, 2.0));
        // Deterministic: same inputs, same normal (no jitter).
        let m2 = poly_poly(&a, &b).expect("overlap");
        assert!(close_vec(m.normal, m2.normal));
    }

    #[test]
    fn poly_circle_face_normal() {
        let poly = square(0.0, 0.0, 1.0);
        let m = poly_circle(&poly, Vec2::new(1.5, 0.0), 1.0).expect("overlap");
        assert!(close_vec(m.normal, Vec2::X), "normal {:?}", m.normal);
        assert!(close(m.depth, 0.5), "depth {}", m.depth);
    }

    #[test]
    fn poly_circle_corner_normal() {
        let poly = square(0.0, 0.0, 1.0);
        // Center diagonally off the (1,1) corner, radius reaches past it.
        let m = poly_circle(&poly, Vec2::new(2.0, 2.0), 1.5).expect("overlap");
        let diag = Vec2::splat(1.0).normalize();
        assert!(close_vec(m.normal, diag), "normal {:?}", m.normal);
        assert!(close(m.depth, 1.5 - 2.0_f32.sqrt()), "depth {}", m.depth);
    }

    #[test]
    fn circle_circle_depth_and_normal() {
        let m = circle_circle(Vec2::ZERO, 1.0, Vec2::new(1.5, 0.0), 1.0).expect("overlap");
        assert!(close_vec(m.normal, Vec2::X));
        assert!(close(m.depth, 0.5));
        assert!(circle_circle(Vec2::ZERO, 1.0, Vec2::new(2.0, 0.0), 1.0).is_none());
    }

    #[test]
    fn circle_circle_coincident_fallback() {
        let m = circle_circle(Vec2::ZERO, 1.0, Vec2::ZERO, 1.0).expect("overlap");
        assert!(close_vec(m.normal, Vec2::X));
        assert!(close(m.depth, 2.0));
    }
}
