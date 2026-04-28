use bevy::prelude::*;

use crate::collision::components::{Collider, Solid, Static};
use crate::collision::events::CollisionEvent;

pub fn resolve_solid_collisions(
    mut reader: MessageReader<CollisionEvent>,
    solids: Query<(), With<Solid>>,
    statics: Query<(), With<Static>>,
    colliders: Query<&Collider>,
    mut transforms: Query<&mut Transform>,
) {
    for ev in reader.read() {
        let (a, b) = (ev.a, ev.b);

        if !(solids.contains(a) && solids.contains(b)) {
            continue;
        }

        let Ok(a_col) = colliders.get(a) else { continue };
        let Ok(b_col) = colliders.get(b) else { continue };
        let half_sum = a_col.half_extents + b_col.half_extents;

        let (a_center, b_center) = match (transforms.get(a), transforms.get(b)) {
            (Ok(a_tx), Ok(b_tx)) => (a_tx.translation.truncate(), b_tx.translation.truncate()),
            _ => continue,
        };

        let delta = a_center - b_center;
        let overlap = half_sum - delta.abs();
        if overlap.x <= 0.0 || overlap.y <= 0.0 {
            continue;
        }

        let push = if overlap.x < overlap.y {
            Vec2::new(overlap.x * delta.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, overlap.y * delta.y.signum())
        };

        let a_static = statics.contains(a);
        let b_static = statics.contains(b);

        match (a_static, b_static) {
            (true, true) => {}
            (true, false) => {
                if let Ok(mut t) = transforms.get_mut(b) {
                    t.translation.x -= push.x;
                    t.translation.y -= push.y;
                }
            }
            (false, true) => {
                if let Ok(mut t) = transforms.get_mut(a) {
                    t.translation.x += push.x;
                    t.translation.y += push.y;
                }
            }
            (false, false) => {
                if let Ok(mut t) = transforms.get_mut(a) {
                    t.translation.x += push.x * 0.5;
                    t.translation.y += push.y * 0.5;
                }
                if let Ok(mut t) = transforms.get_mut(b) {
                    t.translation.x -= push.x * 0.5;
                    t.translation.y -= push.y * 0.5;
                }
            }
        }
    }
}
