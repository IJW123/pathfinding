use bevy::prelude::*;

use collision::plugin::CollisionPlugin;
use hud::plugin::HudPlugin;
use player::plugin::PlayerPlugin;
use render::plugin::RenderPlugin;
use world::elevation::plugin::ElevationPlugin;
use world::plugin::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            CollisionPlugin,
            PlayerPlugin,
            WorldPlugin,
            ElevationPlugin,
            HudPlugin,
            RenderPlugin,
        ))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
