use bevy::prelude::*;

use crate::systems::measure_velocity;

/// System set wrapping [`measure_velocity`]. Exists so consumers can order
/// against it.
///
/// # Scheduling contract
///
/// `motion` is a leaf crate and cannot enforce this itself — the ordering
/// must be configured by the app that wires the plugins together:
///
/// ```text
/// <all movement + resolution systems whose displacement should count>
///   -> MotionSet
///   -> consumers (e.g. HUD readouts)
/// ```
///
/// Adding [`MotionPlugin`] without ordering `MotionSet` after your
/// movement and resolution systems yields stale or wrong readings (the
/// measured delta lags a tick, or misses collision push-out). The current
/// app wires it correctly, all in `FixedUpdate`:
/// `move_player -> CollisionSet -> MotionSet`. Cross-schedule consumers
/// (e.g. HUD in `Update`) read the last completed tick's value.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MotionSet;

/// Measures [`MeasuredVelocity`](crate::components::MeasuredVelocity) from
/// `Transform` deltas each fixed tick. See [`MotionSet`] for the ordering
/// the app must enforce.
pub struct MotionPlugin;

impl Plugin for MotionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, measure_velocity.in_set(MotionSet));
    }
}
