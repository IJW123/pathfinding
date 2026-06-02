use bevy::camera::ScalingMode;
use bevy::prelude::*;

use crate::constants::{CAMERA_PAN_SPEED, DEFAULT_VIEW_HEIGHT_M, ZOOM_MAX, ZOOM_MIN, ZOOM_SPEED};

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: DEFAULT_VIEW_HEIGHT_M,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
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

/// Hold `R` to zoom in (shrink visible metres) and `F` to zoom out, scaling the
/// orthographic projection exponentially so the rate feels constant at any zoom.
pub fn zoom_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut projection: Single<&mut Projection, With<Camera2d>>,
) {
    if let Projection::Orthographic(ortho) = projection.as_mut() {
        let mut factor = 1.0;
        if keyboard.pressed(KeyCode::KeyR) {
            factor /= ZOOM_SPEED.powf(time.delta_secs());
        }
        if keyboard.pressed(KeyCode::KeyF) {
            factor *= ZOOM_SPEED.powf(time.delta_secs());
        }
        ortho.scale = (ortho.scale * factor).clamp(ZOOM_MIN, ZOOM_MAX);
    }
}
