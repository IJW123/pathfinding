use bevy::prelude::*;

use crate::constants::CAMERA_PAN_SPEED;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn pan_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let delta = direction.normalize_or_zero() * CAMERA_PAN_SPEED * time.delta_secs();
    camera.translation += delta.extend(0.0);
}
