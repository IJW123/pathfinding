use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use obstacle::bundle::{boundary_walls, pushable_obstacle, static_obstacle};
use obstacle::constants::{OBSTACLE_Z, WALL_THICKNESS};
use obstacle::shape::{circle, pentagon, quad, triangle};

use crate::objects::manifest::ObstacleShape;
use crate::objects::player::carrier_player;
use crate::objects::spec::{LevelSpec, ObstacleSpec};
use crate::objects::storage::storage;

/// The single place the starting world is populated, from the authored [`LevelSpec`]. Per-instance
/// layout (position/size/shape) is data in `assets/level.ron`; each object's full component
/// composition lives with its constructor in [`crate::objects`]. The boundary walls are derived from
/// the map's `map_half_extent` here — parameterized by map size, never authored per wall.
pub fn spawn_level(mut commands: Commands, level: Res<LevelSpec>) {
    for wall in boundary_walls(level.map_half_extent, WALL_THICKNESS) {
        commands.spawn(wall);
    }

    for obstacle in &level.obstacles {
        let transform = Transform::from_xyz(obstacle.pos.x, obstacle.pos.y, OBSTACLE_Z)
            .with_rotation(Quat::from_rotation_z(obstacle.rotation));
        let collider = obstacle_collider(obstacle);
        if obstacle.pushable {
            commands.spawn(pushable_obstacle(transform, collider));
        } else {
            commands.spawn(static_obstacle(transform, collider));
        }
    }

    commands.spawn(storage(&level.storage));
    commands.spawn(carrier_player(&level.carrier));
}

/// The collider for an obstacle's silhouette, scaled to its authored size. The `obstacle` crate
/// owns the silhouettes; this just routes the spec's [`ObstacleShape`] to the matching constructor.
fn obstacle_collider(spec: &ObstacleSpec) -> Collider {
    match spec.shape {
        ObstacleShape::Circle => circle(spec.size),
        ObstacleShape::Triangle => triangle(spec.size),
        ObstacleShape::Quad => quad(spec.size),
        ObstacleShape::Pentagon => pentagon(spec.size),
    }
}
