use bevy::math::Vec2;

use crate::terrain_effects::constants::{SLOPE_SPEED_K, SLOPE_SPEED_MAX, SLOPE_SPEED_MIN};

#[must_use]
pub fn slope_speed_multiplier(dir: Vec2, gradient: Vec2) -> f32 {
    (1.0 - SLOPE_SPEED_K * dir.dot(gradient)).clamp(SLOPE_SPEED_MIN, SLOPE_SPEED_MAX)
}
