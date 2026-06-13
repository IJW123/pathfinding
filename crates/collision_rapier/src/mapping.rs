use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use rapier2d::geometry::ColliderHandle;

/// Rapier handle owned by one entity. Every collider is parentless (rapier treats it as
/// fixed): `CollisionPipeline` skips the island bookkeeping that rapier's narrow phase
/// asserts on when a contact starts between two rigid bodies, so attaching bodies to
/// dynamics would panic on their first dynamic-dynamic contact. Dynamics are instead plain
/// colliders whose pose is rewritten every tick and which opt into `FIXED_FIXED` contacts.
pub enum BodyBinding {
    Static { collider: ColliderHandle },
    Dynamic { collider: ColliderHandle },
}

impl BodyBinding {
    #[must_use]
    pub fn is_static(&self) -> bool {
        matches!(self, Self::Static { .. })
    }

    #[must_use]
    pub fn collider(&self) -> ColliderHandle {
        match self {
            Self::Static { collider } | Self::Dynamic { collider } => *collider,
        }
    }
}

/// Entity → rapier-handle bindings, maintained by `sync_physics_world`. The map is the ground
/// truth for liveness: an entity present here but gone from the ECS gets its handle removed
/// next tick (skip-proof, unlike removal messages).
#[derive(Resource, Default)]
pub struct ColliderMap {
    pub bindings: HashMap<Entity, BodyBinding>,
}

/// Entity bits round-trip through `Collider::user_data` so contact pairs map back to entities.
#[must_use]
pub fn entity_to_user_data(entity: Entity) -> u128 {
    u128::from(entity.to_bits())
}

#[must_use]
pub fn entity_from_user_data(user_data: u128) -> Entity {
    Entity::from_bits(u64::try_from(user_data).expect("user_data written by entity_to_user_data"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_user_data_roundtrip() {
        let entity = Entity::from_bits(0x0000_0001_0000_002a);
        assert_eq!(entity_from_user_data(entity_to_user_data(entity)), entity);
    }
}
