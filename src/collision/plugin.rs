use bevy::prelude::*;

use crate::collision::events::CollisionEvent;
use crate::collision::narrow_phase::detect_collisions;
use crate::collision::response::resolve_solid_collisions;
use crate::collision::spatial_hash::{SpatialHash, rebuild_spatial_hash};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionSet;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialHash>()
            .add_message::<CollisionEvent>()
            .add_systems(
                Update,
                (
                    rebuild_spatial_hash,
                    detect_collisions,
                    resolve_solid_collisions,
                )
                    .chain()
                    .in_set(CollisionSet),
            );
    }
}
