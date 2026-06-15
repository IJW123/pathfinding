//! Integration test exercising the library boundary: `spawn_level` runs in a minimal headless
//! `App` (no `DefaultPlugins`, no render). This would be impossible if placement lived in `app`'s
//! binary — which is the concrete reason `level` is its own crate.

use bevy::prelude::*;

use hitboxes_rapier::components::Static;
use level::plugin::LevelPlugin;
use obstacle::components::{Obstacle, Wall};
use player::components::Player;

fn run_level() -> App {
    let mut app = App::new();
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
