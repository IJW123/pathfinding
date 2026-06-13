use bevy::prelude::*;

use crate::constants::CONTACT_PREDICTION_DISTANCE;
use crate::physics::PhysicsWorld;

/// One rapier collision-detection step: BVH broad phase + narrow phase, contact pairs land in
/// `PhysicsWorld::narrow_phase`. This call replaces the old grid, static index, AABB and SAT
/// machinery wholesale.
pub fn step_collision_pipeline(mut physics: ResMut<PhysicsWorld>) {
    let PhysicsWorld {
        bodies,
        colliders,
        islands,
        broad_phase,
        narrow_phase,
        pipeline,
    } = &mut *physics;
    pipeline.step(
        CONTACT_PREDICTION_DISTANCE,
        islands,
        broad_phase,
        narrow_phase,
        bodies,
        colliders,
        &(),
        &(),
    );
}
