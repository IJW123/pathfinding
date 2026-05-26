use bevy::math::Vec2;

const SLOPE_SPEED_K: f32 = 0.6;
const SLOPE_SPEED_MIN: f32 = 0.2;
const SLOPE_SPEED_MAX: f32 = 1.5;

#[must_use]
pub fn slope_speed_multiplier(dir: Vec2, gradient: Vec2) -> f32 {
    (1.0 - SLOPE_SPEED_K * dir.dot(gradient)).clamp(SLOPE_SPEED_MIN, SLOPE_SPEED_MAX)
}