use bevy::prelude::*;

use crate::components::Player;
use crate::constants::PLAYER_SPEED;
use world::elevation::height_field::HeightField;
use world::terrain_effects::slope_speed::slope_speed_multiplier;

pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    height: Res<HeightField>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        let dir = direction.normalize();
        for mut transform in &mut query {
            let grad = height.gradient(transform.translation.xy());
            let slope_mul = slope_speed_multiplier(dir, grad);
            let delta = dir * PLAYER_SPEED * slope_mul * time.delta_secs();
            transform.translation.x += delta.x;
            transform.translation.y += delta.y;
        }
    }
}
