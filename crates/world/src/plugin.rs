use bevy::prelude::*;

use crate::wall::spawn_bounds;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_bounds);
    }
}
