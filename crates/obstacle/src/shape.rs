//! Form-only shape templates. Each silhouette is stored at **unit size** (circumradius 1.0),
//! local-space, origin-centered; the public constructors scale it to a world size. Size and
//! placement (center/rotation) are the caller's decision — see the `level` crate.

use bevy::math::Vec2;

use hitboxes_rapier::components::Collider;

/// Unit triangle silhouette: circumradius 1.0, CCW, origin-centered.
const TRIANGLE_UNIT: [Vec2; 3] = [
    Vec2::new(-0.800, -0.600),
    Vec2::new(0.933, -0.333),
    Vec2::new(-0.133, 0.933),
];

/// Unit irregular convex quad: circumradius 1.0, CCW, origin-centered.
const QUAD_UNIT: [Vec2; 4] = [
    Vec2::new(-0.740, -0.538),
    Vec2::new(0.673, -0.740),
    Vec2::new(0.875, 0.471),
    Vec2::new(-0.538, 0.673),
];

/// Unit pentagon: circumradius 1.0, CCW, origin-centered.
const PENTAGON_UNIT: [Vec2; 5] = [
    Vec2::new(0.000, 0.997),
    Vec2::new(-0.951, 0.307),
    Vec2::new(-0.583, -0.813),
    Vec2::new(0.583, -0.813),
    Vec2::new(0.951, 0.307),
];

/// Scale a unit silhouette to `size` and wrap it in a convex hull. The unit templates span a
/// nonzero area, so any `size > 0` yields a valid hull — the `expect` guards an authoring mistake
/// in the unit constants, not caller input.
#[must_use]
fn scaled_hull(unit: &[Vec2], size: f32) -> Collider {
    let points = unit.iter().map(|p| *p * size).collect();
    Collider::convex(points).expect("scaled unit silhouette is a valid convex hull")
}

/// Triangle collider of circumradius `size`, centered on the entity origin.
#[must_use]
pub fn triangle(size: f32) -> Collider {
    scaled_hull(&TRIANGLE_UNIT, size)
}

/// Irregular convex quad collider of circumradius `size`, centered on the entity origin.
#[must_use]
pub fn quad(size: f32) -> Collider {
    scaled_hull(&QUAD_UNIT, size)
}

/// Pentagon collider of circumradius `size`, centered on the entity origin.
#[must_use]
pub fn pentagon(size: f32) -> Collider {
    scaled_hull(&PENTAGON_UNIT, size)
}

/// Circle collider of the given radius. Thin re-expose of [`Collider::circle`] so all obstacle
/// shapes read off one module at the call site.
#[must_use]
pub fn circle(radius: f32) -> Collider {
    Collider::circle(radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygon_hull_vertex_counts() {
        assert_eq!(triangle(50.0).shape.hull_points().expect("convex").len(), 3);
        assert_eq!(quad(50.0).shape.hull_points().expect("convex").len(), 4);
        assert_eq!(pentagon(50.0).shape.hull_points().expect("convex").len(), 5);
    }

    #[test]
    fn size_scales_render_extent_linearly() {
        let small = triangle(50.0).render_size();
        let big = triangle(100.0).render_size();
        assert!(
            (big - small * 2.0).length() < 1e-3,
            "doubling size doubles span"
        );
    }
}
