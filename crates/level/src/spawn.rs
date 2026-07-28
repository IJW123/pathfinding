use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use logistics::components::Capacity;
use obstacle::bundle::{boundary_walls, pushable_obstacle, static_obstacle};
use obstacle::constants::{OBSTACLE_Z, WALL_THICKNESS};
use obstacle::shape::{circle, pentagon, quad, triangle};
use rail::spawn::spawn_rail_line;
use rail::track::RailTrack;
use station::bundle::cargo_train;
use station::constants::{TRAIN_MAX_VOLUME, TRAIN_MAX_WEIGHT};

use crate::objects::manifest::ObstacleShape;
use crate::objects::player::carrier_player;
use crate::objects::spec::{LevelSpec, ObstacleSpec};
use crate::objects::station::terminal_stations;
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

    for spec in &level.storages {
        commands.spawn(storage(spec));
    }

    commands.spawn(carrier_player(&level.carrier));

    // Each authored rail line: a track plus the cargo train riding it. `spawn_rail_line` owns the
    // borrow-vs-move spawn ordering and binds the loco to its own track via `OnTrack`; we then turn
    // that loco into a cargo train so the station broker can load/unload it. Stations are derived,
    // not authored: each track terminal with a warehouse in reach earns one — placed while the
    // track is still borrowable, before `spawn_rail_line` consumes it.
    let storage_positions: Vec<Vec2> = level.storages.iter().map(|spec| spec.pos).collect();
    for spec in &level.rails {
        let track = RailTrack::new(spec.points.clone());
        for bundle in terminal_stations(&track, &storage_positions) {
            commands.spawn(bundle);
        }
        let line = spawn_rail_line(&mut commands, track, spec.start, spec.heading);
        commands
            .entity(line.loco)
            .insert(cargo_train(train_capacity()));
    }
}

/// The hold limits of an authored rail train, from the `station` crate's per-kind constants.
fn train_capacity() -> Capacity {
    Capacity {
        max_weight: Some(TRAIN_MAX_WEIGHT),
        max_volume: Some(TRAIN_MAX_VOLUME),
    }
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
