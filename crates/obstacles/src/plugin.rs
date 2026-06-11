use bevy::prelude::*;

use crate::spawn::setup_obstacles;

pub struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_obstacles);
    }
}
