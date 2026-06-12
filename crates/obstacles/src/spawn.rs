use bevy::prelude::*;

use hitboxes::components::{Collider, Solid, Static};

use crate::components::Obstacle;
use crate::constants::{
    CIRCLE_CENTER, CIRCLE_RADIUS, OBSTACLE_Z, PENTAGON_CENTER, PENTAGON_POINTS, QUAD_CENTER,
    QUAD_POINTS, TRIANGLE_CENTER, TRIANGLE_POINTS, TRIANGLE_TILT_RADIANS,
};

/// Spawn the placed obstacles: a static circle, a static tilted triangle, and two pushable
/// hulls (quad + pentagon). The `expect`s turn hull-authoring mistakes into a descriptive
/// startup panic.
pub fn setup_obstacles(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(CIRCLE_CENTER.x, CIRCLE_CENTER.y, OBSTACLE_Z),
        Obstacle,
        Collider::circle(CIRCLE_RADIUS),
        Solid,
        Static,
    ));
    commands.spawn((
        Transform::from_xyz(TRIANGLE_CENTER.x, TRIANGLE_CENTER.y, OBSTACLE_Z)
            .with_rotation(Quat::from_rotation_z(TRIANGLE_TILT_RADIANS)),
        Obstacle,
        Collider::convex(TRIANGLE_POINTS.to_vec()).expect("valid triangle hull"),
        Solid,
        Static,
    ));
    commands.spawn((
        Transform::from_xyz(QUAD_CENTER.x, QUAD_CENTER.y, OBSTACLE_Z),
        Obstacle,
        Collider::convex(QUAD_POINTS.to_vec()).expect("valid quad hull"),
        Solid,
    ));
    commands.spawn((
        Transform::from_xyz(PENTAGON_CENTER.x, PENTAGON_CENTER.y, OBSTACLE_Z),
        Obstacle,
        Collider::convex(PENTAGON_POINTS.to_vec()).expect("valid pentagon hull"),
        Solid,
    ));
}
