use crate::manifold::Manifold;
use crate::sat::{circle_circle, poly_circle, poly_poly};
use crate::world::WorldShape;

/// Narrow-phase dispatch on world-space geometry. Returns a manifold whose normal points from
/// `a` toward `b`. Circle/Poly reuses `poly_circle` with the normal negated to restore the
/// a→b convention.
#[must_use]
pub fn test_world_pair(a: &WorldShape, b: &WorldShape) -> Option<Manifold> {
    match (a, b) {
        (WorldShape::Poly(a), WorldShape::Poly(b)) => poly_poly(a, b),
        (WorldShape::Poly(a), WorldShape::Circle(c, r)) => poly_circle(a, *c, *r),
        (WorldShape::Circle(c, r), WorldShape::Poly(b)) => {
            poly_circle(b, *c, *r).map(|m| Manifold {
                normal: -m.normal,
                depth: m.depth,
            })
        }
        (WorldShape::Circle(ca, ra), WorldShape::Circle(cb, rb)) => {
            circle_circle(*ca, *ra, *cb, *rb)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::to_world;
    use bevy::prelude::*;
    use hitboxes::shape::ColliderShape;

    #[test]
    fn circle_poly_swap_negates_normal() {
        let circle = to_world(
            &ColliderShape::Circle { radius: 1.0 },
            &Transform::from_xyz(0.0, 0.0, 0.0),
        );
        let poly = to_world(
            &ColliderShape::Obb {
                half_extents: Vec2::splat(1.0),
            },
            &Transform::from_xyz(1.5, 0.0, 0.0),
        );

        let m1 = test_world_pair(&circle, &poly).expect("overlap");
        let m2 = test_world_pair(&poly, &circle).expect("overlap");

        // Opposite normals (a→b vs b→a), equal depth.
        assert!(
            (m1.normal + m2.normal).length() < 1e-3,
            "{:?} {:?}",
            m1.normal,
            m2.normal
        );
        assert!((m1.depth - m2.depth).abs() < 1e-3);
        assert!(
            m1.normal.x > 0.0,
            "circle→poly points right, {:?}",
            m1.normal
        );
    }
}
