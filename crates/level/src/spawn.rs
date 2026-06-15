use bevy::prelude::*;

use obstacle::bundle::{boundary_walls, pushable_obstacle, static_obstacle};
use obstacle::constants::{OBSTACLE_Z, WALL_THICKNESS};
use obstacle::shape::{circle, pentagon, quad, triangle};
use player::bundle::player;

use crate::constants::{CIRCLE_RADIUS, MAP_HALF_EXTENT, PENTAGON_SIZE, QUAD_SIZE, TRIANGLE_SIZE};

/// The single place the starting world is populated. This fn *is* the level layout: per-instance
/// position (`Transform`) and size live here, while the silhouette of each shape lives with its
/// kind in the `obstacle` crate.
pub fn spawn_level(mut commands: Commands) {
    for wall in boundary_walls(MAP_HALF_EXTENT, WALL_THICKNESS) {
        commands.spawn(wall);
    }

    // Static circle.
    commands.spawn(static_obstacle(
        Transform::from_xyz(250.0, 0.0, OBSTACLE_Z),
        circle(CIRCLE_RADIUS),
    ));
    // Static triangle, tilted to exercise rotated collision (0.6 rad ≈ 34°).
    commands.spawn(static_obstacle(
        Transform::from_xyz(280.0, 160.0, OBSTACLE_Z).with_rotation(Quat::from_rotation_z(0.6)),
        triangle(TRIANGLE_SIZE),
    ));
    // Pushable quad.
    commands.spawn(pushable_obstacle(
        Transform::from_xyz(150.0, -260.0, OBSTACLE_Z),
        quad(QUAD_SIZE),
    ));
    // Pushable pentagon.
    commands.spawn(pushable_obstacle(
        Transform::from_xyz(320.0, -200.0, OBSTACLE_Z),
        pentagon(PENTAGON_SIZE),
    ));

    // Player.
    commands.spawn(player(Vec2::ZERO));
}
