use bevy::prelude::*;

use camera_main::plugin::CameraMainPlugin;
use collision_rapier::plugin::{CollisionPlugin, CollisionSet};
use hud::plugin::HudPlugin;
use level::plugin::LevelPlugin;
use motion::plugin::{MotionPlugin, MotionSet};
use player::plugin::PlayerPlugin;
use render::plugin::RenderPlugin;
use world::elevation::plugin::ElevationPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            CameraMainPlugin,
            CollisionPlugin,
            MotionPlugin,
            PlayerPlugin,
            ElevationPlugin,
            LevelPlugin,
            HudPlugin,
            RenderPlugin,
        ))
        .configure_sets(FixedUpdate, MotionSet.after(CollisionSet))
        .run();
}
