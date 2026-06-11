use bevy::prelude::*;

/// A detected overlap. `normal` is a unit vector pointing from `a` toward `b`; `depth` is the
/// penetration along it. The normal is computed for *these* `a,b` (event order) — see
/// `narrow_phase::detect_collisions`.
#[derive(Message)]
pub struct CollisionEvent {
    pub a: Entity,
    pub b: Entity,
    pub normal: Vec2,
    pub depth: f32,
}
