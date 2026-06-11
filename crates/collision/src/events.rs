use bevy::prelude::*;

/// A detected overlap, emitted from the solver's pass-0 (pre-resolution) manifolds. `normal`
/// is a unit vector pointing from `a` toward `b`; `depth` is the penetration along it, as it
/// was *before* any correction this tick.
///
/// Consumers must read in `FixedUpdate` (after `CollisionSet`): collision runs there, and an
/// `Update`-schedule reader can observe the same message buffer twice on frames that run zero
/// fixed ticks.
#[derive(Message)]
pub struct CollisionEvent {
    pub a: Entity,
    pub b: Entity,
    pub normal: Vec2,
    pub depth: f32,
}
