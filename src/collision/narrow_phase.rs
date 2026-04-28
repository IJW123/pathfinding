use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::collision::components::{Collider, Static};
use crate::collision::events::CollisionEvent;
use crate::collision::spatial_hash::SpatialHash;

pub fn aabb_overlaps(a_center: Vec2, a_half: Vec2, b_center: Vec2, b_half: Vec2) -> bool {
    let delta = (a_center - b_center).abs();
    let total = a_half + b_half;
    delta.x < total.x && delta.y < total.y
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
                let pair = if a.index() < b.index() { (a, b) } else { (b, a) };
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
                if aabb_overlaps(
                    a_tx.translation.truncate(),
                    a_col.half_extents,
                    b_tx.translation.truncate(),
                    b_col.half_extents,
                ) {
                    writer.write(CollisionEvent { a, b });
                }
            }
        }
    }
}
