use bevy::prelude::*;

use camera_main::plugin::CameraMainPlugin;
use collision_rapier::plugin::{CollisionPlugin, CollisionSet};
use hud::plugin::HudPlugin;
use level::plugin::LevelPlugin;
use logistics::plugin::LogisticsPlugin;
use motion::plugin::{MotionPlugin, MotionSet};
use render::plugin::RenderPlugin;
use selection::plugin::SelectionPlugin;
use world::elevation::plugin::ElevationPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            CameraMainPlugin,
            CollisionPlugin,
            MotionPlugin,
            SelectionPlugin,
            ElevationPlugin,
            LogisticsPlugin,
            LevelPlugin,
            HudPlugin,
            RenderPlugin,
        ))
        .configure_sets(FixedUpdate, MotionSet.after(CollisionSet))
        .run();
}
