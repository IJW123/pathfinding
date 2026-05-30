use bevy::prelude::*;

/// Resolved world displacement per second, in world units, observed from
/// the entity's `Transform` delta over the previous frame.
///
/// This is *measured*, not requested: it folds in everything that mutated
/// the `Transform` before [`MotionSet`](crate::plugin::MotionSet) ran —
/// player input, terrain-modified movement, collision push-out, and any
/// scripted transform writes. It is **not** input/requested speed.
///
/// Read-only output of [`MotionPlugin`](crate::plugin::MotionPlugin):
/// writes to this component are overwritten every frame.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct MeasuredVelocity(pub Vec2);

/// Last frame's xy translation. Used by `measure_velocity` to compute
/// the position delta. Initialize to the spawn position to avoid a
/// first-frame velocity spike.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct PrevPosition(pub Vec2);
