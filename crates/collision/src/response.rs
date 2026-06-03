use bevy::prelude::*;

use crate::components::{Collider, Solid, Static};
use crate::events::CollisionEvent;

pub fn resolve_solid_collisions(
    mut reader: MessageReader<CollisionEvent>,
    solids: Query<(), With<Solid>>,
    statics: Query<(), With<Static>>,
    colliders: Query<&Collider>,
    mut transforms: Query<&mut Transform>,
) {
    for ev in reader.read() {
        let CollisionEvent { a, b } = *ev;

        if !(solids.contains(a) && solids.contains(b)) {
            continue;
        }

        let Ok(a_col) = colliders.get(a) else {
            continue;
        };
        let Ok(b_col) = colliders.get(b) else {
            continue;
        };

        let (a_center, b_center) = match (transforms.get(a), transforms.get(b)) {
            (Ok(a_tx), Ok(b_tx)) => (a_tx.translation.truncate(), b_tx.translation.truncate()),
            _ => continue,
        };

        let Some(push) = aabb_mtv(a_center, b_center, a_col.half_extents + b_col.half_extents)
        else {
            continue;
        };

        let (a_factor, b_factor) = match (statics.contains(a), statics.contains(b)) {
            (true, true) => continue,
            (true, false) => (0.0, -1.0),
            (false, true) => (1.0, 0.0),
            (false, false) => (0.5, -0.5),
        };

        let mut apply = |entity: Entity, factor: f32| {
            if factor == 0.0 {
                return;
            }
            if let Ok(mut t) = transforms.get_mut(entity) {
                t.translation.x += push.x * factor;
                t.translation.y += push.y * factor;
            }
        };
        apply(a, a_factor);
        apply(b, b_factor);
    }
}

/// Axis-Aligned Bounding Box Minimum Translation Vector: shortest push that separates two overlapping AABBs, or `None` if they don't overlap.
#[must_use]
fn aabb_mtv(a_center: Vec2, b_center: Vec2, half_sum: Vec2) -> Option<Vec2> {
    let delta = a_center - b_center;
    let overlap = half_sum - delta.abs();
    (overlap.x > 0.0 && overlap.y > 0.0).then(|| {
        if overlap.x < overlap.y {
            Vec2::new(overlap.x * delta.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, overlap.y * delta.y.signum())
        }
    })
}
