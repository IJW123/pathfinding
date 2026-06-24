use bevy::prelude::*;

use crate::systems::{drive_locomotive, project_locomotive, turn_locomotive};

/// Wires the rail/locomotive systems. `turn_locomotive` reads edge-detected input in `Update`;
/// driving and projection run as an ordered pair each `FixedUpdate` tick (drive writes the
/// arc-length, project derives the pose from it).
///
/// No ordering vs `CollisionSet` is needed for correctness: the loco is non-`Solid`, so the solver
/// never moves it, and it carries no `MeasuredVelocity`. Spawning the track + locomotive is the
/// `level` crate's job (it owns the authored data), so this plugin only adds behavior.
pub struct RailPlugin;

impl Plugin for RailPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, turn_locomotive)
            .add_systems(FixedUpdate, (drive_locomotive, project_locomotive).chain());
    }
}
