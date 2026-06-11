use bevy::prelude::*;

use crate::broad_phase::{CollisionPairs, DynamicBodies, collect_collision_pairs};
use crate::events::CollisionEvent;
use crate::solver::resolve_collisions;
use crate::static_index::{StaticColliderIndex, maintain_static_index};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionSet;

/// Collision runs in `FixedUpdate`: a fixed tick bounds the per-step displacement
/// (max speed ÷ tick rate), which is what makes thin walls tunnel-proof.
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StaticColliderIndex>()
            .init_resource::<DynamicBodies>()
            .init_resource::<CollisionPairs>()
            .add_message::<CollisionEvent>()
            .add_systems(
                FixedUpdate,
                (
                    maintain_static_index,
                    collect_collision_pairs,
                    resolve_collisions,
                )
                    .chain()
                    .in_set(CollisionSet),
            );
    }
}
