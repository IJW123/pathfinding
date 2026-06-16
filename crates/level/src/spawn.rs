use bevy::prelude::*;

use logistics::bundle::storage_building;
use logistics::cargo_handling::components::{Carrier, DockZone};
use logistics::commodity::Commodity;
use logistics::components::{Capacity, Inventory};
use logistics::constants::STORAGE_Z;
use obstacle::bundle::{boundary_walls, pushable_obstacle, static_obstacle};
use obstacle::constants::{OBSTACLE_Z, WALL_THICKNESS};
use obstacle::shape::{circle, pentagon, quad, triangle};
use player::bundle::player;

use crate::constants::{
    CARRIER_MAX_VOLUME, CARRIER_MAX_WEIGHT, CIRCLE_RADIUS, MAP_HALF_EXTENT, PENTAGON_SIZE,
    QUAD_SIZE, STORAGE_DOCK_RADIUS, STORAGE_HALF_EXTENT, STORAGE_MAX_VOLUME, TRIANGLE_SIZE,
};

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

    // Storage building (infrastructure): a square holding a starting stock of goods. Capped by
    // space only, with a circular dock zone a carrier must enter to load/unload.
    commands
        .spawn(storage_building(
            Transform::from_xyz(-250.0, 200.0, STORAGE_Z),
            Vec2::splat(STORAGE_HALF_EXTENT),
            Inventory::from_stock([
                (Commodity::Grain, 100),
                (Commodity::Coal, 40),
                (Commodity::Lumber, 60),
                (Commodity::IronOre, 20),
            ]),
        ))
        .insert((
            Capacity {
                max_weight: None,
                max_volume: Some(STORAGE_MAX_VOLUME),
            },
            DockZone {
                radius: STORAGE_DOCK_RADIUS,
            },
        ));

    // Player doubles as the cargo carrier for now: an empty inventory capped on both weight and
    // volume, so hauling a full building clamps.
    commands.spawn(player(Vec2::ZERO)).insert((
        Inventory::default(),
        Carrier,
        Capacity {
            max_weight: Some(CARRIER_MAX_WEIGHT),
            max_volume: Some(CARRIER_MAX_VOLUME),
        },
    ));
}
