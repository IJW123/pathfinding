use bevy::prelude::*;
use rapier2d::geometry::{ActiveCollisionTypes, ColliderBuilder};

use hitboxes_rapier::components::{Collider, Static};
use hitboxes_rapier::convert::transform_to_pose;

use crate::mapping::{BodyBinding, ColliderMap, entity_to_user_data};
use crate::physics::PhysicsWorld;

/// All our colliders are parentless, which rapier classifies as fixed (see [`BodyBinding`]).
/// Dynamic colliders therefore opt into `FIXED_FIXED` contacts. Pair activity is the OR of
/// both colliders' flags, so statics never carry it — and static-static pairs stay
/// structurally impossible.
const DYNAMIC_COLLISION_TYPES: ActiveCollisionTypes = ActiveCollisionTypes::FIXED_FIXED;

fn remove_binding(physics: &mut PhysicsWorld, binding: &BodyBinding) {
    let PhysicsWorld {
        bodies,
        colliders,
        islands,
        ..
    } = physics;
    colliders.remove(binding.collider(), islands, bodies, false);
}

fn insert_binding(
    physics: &mut PhysicsWorld,
    entity: Entity,
    transform: &Transform,
    collider: &Collider,
    is_static: bool,
) -> BodyBinding {
    let mut builder = ColliderBuilder::new(collider.shape.to_shared_shape())
        .position(transform_to_pose(transform))
        .user_data(entity_to_user_data(entity));
    if !is_static {
        builder = builder.active_collision_types(DYNAMIC_COLLISION_TYPES);
    }
    let handle = physics.colliders.insert(builder);
    if is_static {
        BodyBinding::Static { collider: handle }
    } else {
        BodyBinding::Dynamic { collider: handle }
    }
}

/// Statics whose pose must be re-pushed into rapier this tick.
type MovedStatics = (With<Static>, With<Collider>, Changed<Transform>);

/// Mirror the ECS into the rapier sets. Every step is skip-proof for `FixedUpdate` frame
/// skipping: liveness is map-vs-world (never removal messages), insertion is map-presence
/// (never `Added`), and change detection ticks survive skipped frames.
pub fn sync_physics_world(
    mut physics: ResMut<PhysicsWorld>,
    mut map: ResMut<ColliderMap>,
    all: Query<(Entity, &Transform, &Collider, Has<Static>)>,
    changed_shapes: Query<(Entity, &Collider), Changed<Collider>>,
    moved_statics: Query<(Entity, &Transform), MovedStatics>,
) {
    // Despawned/stripped entities, plus bindings whose Static marker flipped (recreated below).
    let stale: Vec<Entity> = map
        .bindings
        .iter()
        .filter_map(|(entity, binding)| match all.get(*entity) {
            Err(_) => Some(*entity),
            Ok((.., is_static)) => (is_static != binding.is_static()).then_some(*entity),
        })
        .collect();
    for entity in stale {
        if let Some(binding) = map.bindings.remove(&entity) {
            remove_binding(&mut physics, &binding);
        }
    }

    for (entity, transform, collider, is_static) in &all {
        if !map.bindings.contains_key(&entity) {
            let binding = insert_binding(&mut physics, entity, transform, collider, is_static);
            map.bindings.insert(entity, binding);
        }
    }

    // `Changed` covers `Added`; re-setting a just-inserted shape is harmless.
    for (entity, collider) in &changed_shapes {
        if let Some(binding) = map.bindings.get(&entity)
            && let Some(rapier_collider) = physics.colliders.get_mut(binding.collider())
        {
            rapier_collider.set_shape(collider.shape.to_shared_shape());
        }
    }

    // Statics move only on explicit Transform changes; dynamics re-sync unconditionally —
    // they move every tick, and this also heals the post-writeback stale poses.
    for (entity, transform) in &moved_statics {
        if let Some(binding @ BodyBinding::Static { .. }) = map.bindings.get(&entity)
            && let Some(rapier_collider) = physics.colliders.get_mut(binding.collider())
        {
            rapier_collider.set_position(transform_to_pose(transform));
        }
    }
    for (entity, transform, _, is_static) in &all {
        if !is_static
            && let Some(binding) = map.bindings.get(&entity)
            && let Some(rapier_collider) = physics.colliders.get_mut(binding.collider())
        {
            rapier_collider.set_position(transform_to_pose(transform));
        }
    }
}
