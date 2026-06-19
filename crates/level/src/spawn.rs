use bevy::prelude::*;

use logistics::commodity::Commodity;
use logistics::components::Inventory;
use logistics::constants::STORAGE_Z;
use obstacle::bundle::{boundary_walls, pushable_obstacle, static_obstacle};
use obstacle::constants::{OBSTACLE_Z, WALL_THICKNESS};
use obstacle::shape::{circle, pentagon, quad, triangle};

use crate::constants::{
    CIRCLE_RADIUS, MAP_HALF_EXTENT, PENTAGON_SIZE, QUAD_SIZE, STORAGE_HALF_EXTENT, TRIANGLE_SIZE,
};
use crate::objects::player::carrier_player;
use crate::objects::storage::storage;

/// The single place the starting world is populated. This fn *is* the level layout: per-instance
/// position (`Transform`) and size live here, while each object's full component composition lives
/// with its constructor in [`crate::objects`].
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

    // Storage building (infrastructure): a square holding a starting stock of goods.
    commands.spawn(storage(
        Transform::from_xyz(-250.0, 200.0, STORAGE_Z),
        STORAGE_HALF_EXTENT,
        Inventory::from_stock([
            (Commodity::Grain, 100),
            (Commodity::Coal, 40),
            (Commodity::Lumber, 60),
            (Commodity::IronOre, 20),
        ]),
    ));

    // Player doubles as the cargo carrier, controlled by default.
    commands.spawn(carrier_player(Vec2::ZERO));
}
