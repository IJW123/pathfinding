use bevy::prelude::*;

/// Narrow-phase contact result. `normal` is a unit vector pointing from `a` toward `b`;
/// `depth` is the penetration along that normal (`>= 0`). Response separates with
/// `push = normal * depth`.
#[derive(Clone, Copy, Debug)]
pub struct Manifold {
    pub normal: Vec2,
    pub depth: f32,
}
