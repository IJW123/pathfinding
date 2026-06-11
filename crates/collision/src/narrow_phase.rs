use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::components::{Collider, Static};
use crate::events::CollisionEvent;
use crate::manifold::Manifold;
use crate::sat::{circle_circle, poly_circle, poly_poly};
use crate::shape::ColliderShape;
use crate::spatial_hash::SpatialHash;
use crate::world::{WorldShape, to_world};

/// Narrow-phase dispatch. Returns a manifold whose normal points from `a` toward `b`.
/// Circle/Poly reuses `poly_circle` with the normal negated to restore the a→b convention.
#[must_use]
pub fn test_pair(
    a_shape: &ColliderShape,
    a_tx: &Transform,
    b_shape: &ColliderShape,
    b_tx: &Transform,
) -> Option<Manifold> {
    match (to_world(a_shape, a_tx), to_world(b_shape, b_tx)) {
        (WorldShape::Poly(a), WorldShape::Poly(b)) => poly_poly(&a, &b),
        (WorldShape::Poly(a), WorldShape::Circle(c, r)) => poly_circle(&a, c, r),
        (WorldShape::Circle(c, r), WorldShape::Poly(b)) => {
            poly_circle(&b, c, r).map(|m| Manifold {
                normal: -m.normal,
                depth: m.depth,
            })
        }
        (WorldShape::Circle(ca, ra), WorldShape::Circle(cb, rb)) => circle_circle(ca, ra, cb, rb),
    }
}

pub fn detect_collisions(
    hash: Res<SpatialHash>,
    query: Query<(&Transform, &Collider, Option<&Static>)>,
    mut writer: MessageWriter<CollisionEvent>,
) {
    let mut seen: HashSet<(Entity, Entity)> = HashSet::default();
    for cell_entities in hash.cells.values() {
        for (i, &a) in cell_entities.iter().enumerate() {
            for &b in &cell_entities[i + 1..] {
                // Index-ordered key dedups the HashSet only — the event keeps loop order so the
                // a→b normal stays correct.
                let pair = if a.index() < b.index() {
                    (a, b)
                } else {
                    (b, a)
                };
                if !seen.insert(pair) {
                    continue;
                }
                let Ok((a_tx, a_col, a_static)) = query.get(a) else {
                    continue;
                };
                let Ok((b_tx, b_col, b_static)) = query.get(b) else {
                    continue;
                };
                if a_static.is_some() && b_static.is_some() {
                    continue;
                }
                if let Some(m) = test_pair(&a_col.shape, a_tx, &b_col.shape, b_tx) {
                    writer.write(CollisionEvent {
                        a,
                        b,
                        normal: m.normal,
                        depth: m.depth,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pair_circle_poly_swap_negates_normal() {
        let circle = ColliderShape::Circle { radius: 1.0 };
        let poly = ColliderShape::Obb {
            half_extents: Vec2::splat(1.0),
        };
        let circle_tx = Transform::from_xyz(0.0, 0.0, 0.0);
        let poly_tx = Transform::from_xyz(1.5, 0.0, 0.0);

        let m1 = test_pair(&circle, &circle_tx, &poly, &poly_tx).expect("overlap");
        let m2 = test_pair(&poly, &poly_tx, &circle, &circle_tx).expect("overlap");

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
