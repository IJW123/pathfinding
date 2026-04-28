use bevy::prelude::*;

use crate::collision::components::{Collider, Solid};
use crate::constants::{PLAYER_SIZE, PLAYER_SPEED};
use crate::player::components::Player;

pub fn setup_player(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.7, 1.0),
            custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player,
        Collider {
            half_extents: Vec2::splat(PLAYER_SIZE / 2.0),
        },
        Solid,
    ));
}

pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
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
        let delta = direction.normalize() * PLAYER_SPEED * time.delta_secs();
        for mut transform in &mut query {
            transform.translation.x += delta.x;
            transform.translation.y += delta.y;
        }
    }
}
