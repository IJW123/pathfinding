use bevy::prelude::*;

use camera_main::plugin::CameraMainPlugin;
use collision::plugin::{CollisionPlugin, CollisionSet};
use hud::plugin::HudPlugin;
use motion::plugin::{MotionPlugin, MotionSet};
use player::plugin::PlayerPlugin;
use render::plugin::RenderPlugin;
use world::elevation::plugin::ElevationPlugin;
use world::plugin::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            CameraMainPlugin,
            CollisionPlugin,
            MotionPlugin,
            PlayerPlugin,
            WorldPlugin,
            ElevationPlugin,
            HudPlugin,
            RenderPlugin,
        ))
        .configure_sets(Update, MotionSet.after(CollisionSet))
        .run();
}
