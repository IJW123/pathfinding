use bevy::prelude::*;
use rapier2d::dynamics::{IslandManager, RigidBodySet};
use rapier2d::geometry::{BroadPhaseBvh, ColliderSet, NarrowPhase};
use rapier2d::pipeline::CollisionPipeline;

/// Rapier collision-detection state. Detection only: `CollisionPipeline` runs broad + narrow
/// phase; resolution stays in our solver. Poses go stale after the solver's Transform
/// writeback each tick; the next tick's sync rewrites every kinematic pose before stepping.
#[derive(Resource, Default)]
pub struct PhysicsWorld {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub islands: IslandManager,
    pub broad_phase: BroadPhaseBvh,
    pub narrow_phase: NarrowPhase,
    pub pipeline: CollisionPipeline,
}
