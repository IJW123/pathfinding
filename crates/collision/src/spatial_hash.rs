use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::components::Collider;
use crate::constants::CELL_SIZE;

#[derive(Resource, Default)]
pub struct SpatialHash {
    pub cells: HashMap<IVec2, Vec<Entity>>,
}

impl SpatialHash {
    pub fn cell_of(point: Vec2) -> IVec2 {
        IVec2::new(
            (point.x / CELL_SIZE).floor() as i32,
            (point.y / CELL_SIZE).floor() as i32,
        )
    }

    pub fn cells_for_aabb(center: Vec2, half_extents: Vec2) -> impl Iterator<Item = IVec2> {
        let min = Self::cell_of(center - half_extents);
        let max = Self::cell_of(center + half_extents);
        (min.y..=max.y).flat_map(move |y| (min.x..=max.x).map(move |x| IVec2::new(x, y)))
    }
}

pub fn rebuild_spatial_hash(
    mut hash: ResMut<SpatialHash>,
    query: Query<(Entity, &Transform, &Collider)>,
) {
    hash.cells.clear();
    for (entity, transform, collider) in &query {
        let center = transform.translation.truncate();
        for cell in SpatialHash::cells_for_aabb(center, collider.half_extents) {
            hash.cells.entry(cell).or_default().push(entity);
        }
    }
}
