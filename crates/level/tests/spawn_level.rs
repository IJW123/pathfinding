//! Integration test exercising the library boundary: `spawn_level` runs in a minimal headless
//! `App` (no `DefaultPlugins`, no render). This would be impossible if placement lived in `app`'s
//! binary — which is the concrete reason `level` is its own crate.
//!
//! A `LevelSpec` is inserted before `update()`, so `LevelPlugin`'s `run_if`-guarded loader skips
//! disk (no file IO from the crate dir) and `spawn_level` reads this in-memory layout instead.

use bevy::prelude::*;

use hitboxes_rapier::components::Static;
use level::objects::manifest::ObstacleShape;
use level::objects::spec::{
    CarrierSpec, LevelSpec, ObstacleSpec, RailSpec, StorageSpec, TerrainSpec,
};
use level::plugin::LevelPlugin;
use logistics::commodity::Commodity;
use obstacle::components::{Obstacle, Wall};
use player::components::Player;
use rail::components::{Locomotive, RailHeading};
use world::elevation::config::FeaturePopulation;

/// The asserted topology: 4 interior obstacles (2 pushable), one storage, one carrier. Walls are
/// derived from `map_half_extent` in `spawn_level`, not from this spec; `terrain` is required to
/// construct `LevelSpec` (the plugin builds a `TerrainConfig` from it at build time) but isn't
/// asserted here.
fn sample_level() -> LevelSpec {
    let population = FeaturePopulation {
        count: 1,
        radius_min: 100.0,
        radius_max: 200.0,
        height_min: 10.0,
        height_max: 20.0,
    };
    LevelSpec {
        map_half_extent: 2000.0,
        terrain: TerrainSpec {
            seed: 1,
            hills: population,
            mountains: population,
            authored: vec![],
        },
        obstacles: vec![
            ObstacleSpec {
                shape: ObstacleShape::Circle,
                pos: Vec2::new(250.0, 0.0),
                rotation: 0.0,
                size: 60.0,
                pushable: false,
            },
            ObstacleSpec {
                shape: ObstacleShape::Triangle,
                pos: Vec2::new(280.0, 160.0),
                rotation: 0.6,
                size: 75.0,
                pushable: false,
            },
            ObstacleSpec {
                shape: ObstacleShape::Quad,
                pos: Vec2::new(150.0, -260.0),
                rotation: 0.0,
                size: 74.0,
                pushable: true,
            },
            ObstacleSpec {
                shape: ObstacleShape::Pentagon,
                pos: Vec2::new(320.0, -200.0),
                rotation: 0.0,
                size: 65.0,
                pushable: true,
            },
        ],
        storage: StorageSpec {
            pos: Vec2::new(-250.0, 200.0),
            half_extent: 50.0,
            max_volume: 20.0,
            dock_radius: 120.0,
            stock: vec![(Commodity::Grain, 100)],
        },
        carrier: CarrierSpec {
            spawn: Vec2::ZERO,
            max_weight: 2000.0,
            max_volume: 3.0,
        },
        rail: RailSpec {
            points: vec![
                Vec2::new(-600.0, -400.0),
                Vec2::new(-200.0, -400.0),
                Vec2::new(200.0, -150.0),
            ],
            start: 0.0,
            heading: RailHeading::Forward,
        },
    }
}

fn run_level() -> App {
    let mut app = App::new();
    app.insert_resource(sample_level());
    app.add_plugins(LevelPlugin);
    app.update();
    app
}

#[test]
fn spawns_expected_counts() {
    let mut app = run_level();
    let world = app.world_mut();
    assert_eq!(world.query::<&Wall>().iter(world).count(), 4, "4 walls");
    assert_eq!(
        world.query::<&Obstacle>().iter(world).count(),
        8,
        "4 interior obstacles + 4 walls (walls are obstacles)"
    );
    assert_eq!(world.query::<&Player>().iter(world).count(), 1, "1 player");
    assert_eq!(
        world.query::<&Locomotive>().iter(world).count(),
        1,
        "1 locomotive"
    );
}

#[test]
fn two_obstacles_are_pushable() {
    let mut app = run_level();
    let world = app.world_mut();
    let pushable = world
        .query_filtered::<&Obstacle, Without<Static>>()
        .iter(world)
        .count();
    assert_eq!(pushable, 2, "quad + pentagon lack Static");
}
