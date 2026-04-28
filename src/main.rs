mod collision;
mod constants;
mod player;
mod world;

use bevy::prelude::*;

use crate::collision::plugin::CollisionPlugin;
use crate::player::plugin::PlayerPlugin;
use crate::world::plugin::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((CollisionPlugin, PlayerPlugin, WorldPlugin))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
