use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use obstacles::bundle::{boundary_walls, pushable_obstacle, static_obstacle};
use obstacles::constants::{
    CIRCLE_RADIUS, OBSTACLE_Z, PENTAGON_POINTS, QUAD_POINTS, TRIANGLE_POINTS, WALL_THICKNESS,
};
use player::bundle::player;

use crate::constants::MAP_HALF_EXTENT;

/// The single place the starting world is populated. This fn *is* the level layout: instance
/// positions are inline, while geometry templates (radii, hulls, z-layers) live with their kinds.
/// The `expect`s turn hull-authoring mistakes into a descriptive startup panic.
pub fn spawn_level(mut commands: Commands) {
    for wall in boundary_walls(MAP_HALF_EXTENT, WALL_THICKNESS) {
        commands.spawn(wall);
    }

    // Static circle.
    commands.spawn(static_obstacle(
        Transform::from_xyz(250.0, 0.0, OBSTACLE_Z),
        Collider::circle(CIRCLE_RADIUS),
    ));
    // Static triangle, tilted to exercise rotated collision (0.6 rad ≈ 34°).
    commands.spawn(static_obstacle(
        Transform::from_xyz(-280.0, 160.0, OBSTACLE_Z).with_rotation(Quat::from_rotation_z(0.6)),
        Collider::convex(TRIANGLE_POINTS.to_vec()).expect("valid triangle hull"),
    ));
    // Pushable quad.
    commands.spawn(pushable_obstacle(
        Transform::from_xyz(150.0, -260.0, OBSTACLE_Z),
        Collider::convex(QUAD_POINTS.to_vec()).expect("valid quad hull"),
    ));
    // Pushable pentagon.
    commands.spawn(pushable_obstacle(
        Transform::from_xyz(320.0, -200.0, OBSTACLE_Z),
        Collider::convex(PENTAGON_POINTS.to_vec()).expect("valid pentagon hull"),
    ));

    // Player.
    commands.spawn(player(Vec2::ZERO));
}
